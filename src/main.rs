#![forbid(unsafe_code)]

mod lang;

use lang::{
    detect_language, parse_into, LangId, LangStats, ALL_LANGS, LANG_ALL_MASK, LANG_COUNT,
};
use rustix::buffer::spare_capacity;
use rustix::fd::{AsFd, OwnedFd};
use rustix::ffi::CStr;
use rustix::fs::{
    openat, statx, AtFlags, FileType, Mode, OFlags, RawDir, RawMode, StatxFlags, CWD,
};
use rustix::io::{dup, read, write, Errno};
use rustix::stdio::stdout;
use std::cmp::Ordering;
use std::env;
use std::ffi::OsStr;
use std::fmt::Write as _;
use std::os::unix::ffi::OsStrExt;
use std::process;
use std::sync::{Condvar, Mutex};
use std::thread;
use std::time::Instant;

const BATCH_SIZE: usize = 64;
const BATCH_BUF_SIZE: usize = 8192;

const _: () = assert!(BATCH_SIZE > 0 && BATCH_SIZE <= 1024);
const _: () = assert!(BATCH_BUF_SIZE >= 1024 && BATCH_BUF_SIZE <= u16::MAX as usize);

struct Batch {
    dirfd: Option<OwnedFd>,
    data: [u8; BATCH_BUF_SIZE],
    offsets: [u16; BATCH_SIZE],
    name_offsets: [u16; BATCH_SIZE],
    lens: [u16; BATCH_SIZE],
    langs: [LangId; BATCH_SIZE],
    data_len: usize,
    count: usize,
}

impl Batch {
    fn new() -> Box<Self> {
        Box::new(Self {
            dirfd: None,
            data: [0u8; BATCH_BUF_SIZE],
            offsets: [0u16; BATCH_SIZE],
            name_offsets: [0u16; BATCH_SIZE],
            lens: [0u16; BATCH_SIZE],
            langs: [LangId::C; BATCH_SIZE],
            data_len: 0,
            count: 0,
        })
    }

    #[inline(always)]
    fn reset(&mut self) {
        self.dirfd = None;
        self.data_len = 0;
        self.count = 0;
    }

    #[inline(always)]
    fn append(&mut self, dir: &[u8], name: &[u8], lang: LangId, too_long: &mut bool) -> bool {
        debug_assert!(!name.is_empty());
        debug_assert!(self.count <= BATCH_SIZE);
        debug_assert!(self.data_len <= BATCH_BUF_SIZE);
        let dir_len = dir.len();
        let name_len = name.len();
        let has_slash = dir_len > 0 && dir[dir_len - 1] == b'/';
        let sep_len = usize::from(!has_slash);
        let path_len = dir_len + sep_len + name_len;
        let needed = path_len + 1;
        if needed > BATCH_BUF_SIZE {
            *too_long = true;
            return false;
        }
        *too_long = false;
        if self.count >= BATCH_SIZE || self.data_len + needed > BATCH_BUF_SIZE {
            return false;
        }
        let off = self.data_len;
        self.data[off..off + dir_len].copy_from_slice(dir);
        if !has_slash {
            self.data[off + dir_len] = b'/';
        }
        let name_off = off + dir_len + sep_len;
        self.data[name_off..name_off + name_len].copy_from_slice(name);
        self.data[off + path_len] = 0;

        self.offsets[self.count] = off as u16;
        self.name_offsets[self.count] = name_off as u16;
        self.lens[self.count] = path_len as u16;
        self.langs[self.count] = lang;
        self.data_len = off + needed;
        self.count += 1;
        true
    }

    #[inline(always)]
    fn get_path_cstr(&self, idx: usize) -> &CStr {
        debug_assert!(idx < self.count);
        let off = self.offsets[idx] as usize;
        let len = self.lens[idx] as usize;
        debug_assert!(off + len < self.data_len);
        CStr::from_bytes_with_nul(&self.data[off..off + len + 1]).unwrap()
    }

    #[inline(always)]
    fn get_rel_name_cstr(&self, idx: usize) -> &CStr {
        debug_assert!(idx < self.count);
        let off = self.offsets[idx] as usize;
        let name_off = self.name_offsets[idx] as usize;
        let len = self.lens[idx] as usize;
        debug_assert!(name_off >= off && off + len < self.data_len);
        CStr::from_bytes_with_nul(&self.data[name_off..off + len + 1]).unwrap()
    }
}

enum Task {
    Done,
    Batch(Box<Batch>),
    Dir,
}

struct SchedulerState {
    dirs_arena: Vec<u8>,
    dirs_spans: Vec<(u32, u32)>,
    ready_batches: Vec<Box<Batch>>,
    free_batches: Vec<Box<Batch>>,
    active_workers: usize,
    done: bool,
}

struct WorkScheduler {
    enabled_mask: u32,
    mutex: Mutex<SchedulerState>,
    cond: Condvar,
}

impl WorkScheduler {
    fn new(root: &[u8], enabled_mask: u32) -> Self {
        let mut dirs_arena = Vec::with_capacity(65536);
        dirs_arena.extend_from_slice(root);
        let mut dirs_spans = Vec::with_capacity(256);
        dirs_spans.push((0, root.len() as u32));

        Self {
            enabled_mask,
            mutex: Mutex::new(SchedulerState {
                dirs_arena,
                dirs_spans,
                ready_batches: Vec::with_capacity(64),
                free_batches: Vec::with_capacity(64),
                active_workers: 0,
                done: false,
            }),
            cond: Condvar::new(),
        }
    }

    fn acquire_batch(&self) -> Box<Batch> {
        let mut s = self.mutex.lock().unwrap();
        if let Some(mut b) = s.free_batches.pop() {
            drop(s);
            b.reset();
            return b;
        }
        drop(s);
        Batch::new()
    }

    fn submit_batch_and_swap(&self, full_batch: Box<Batch>) -> Box<Batch> {
        debug_assert!(full_batch.count > 0);
        let mut s = self.mutex.lock().unwrap();
        s.ready_batches.push(full_batch);
        self.cond.notify_one();
        if let Some(mut next) = s.free_batches.pop() {
            drop(s);
            next.reset();
            return next;
        }
        drop(s);
        Batch::new()
    }

    fn complete_batch_and_next(
        &self,
        finished_batch: Option<Box<Batch>>,
        local_batch: &mut Option<Box<Batch>>,
        dir_buf: &mut Vec<u8>,
    ) -> Task {
        let mut s = self.mutex.lock().unwrap();
        if let Some(b) = finished_batch {
            debug_assert!(s.active_workers >= 1);
            s.free_batches.push(b);
            s.active_workers -= 1;
        }

        loop {
            if let Some(b) = s.ready_batches.pop() {
                s.active_workers += 1;
                return Task::Batch(b);
            }
            if let Some((off, len)) = s.dirs_spans.pop() {
                let start = off as usize;
                let end = start + len as usize;
                debug_assert!(end <= s.dirs_arena.len());
                dir_buf.clear();
                dir_buf.extend_from_slice(&s.dirs_arena[start..end]);
                dir_buf.push(0);
                s.dirs_arena.truncate(start);
                s.active_workers += 1;
                return Task::Dir;
            }
            if let Some(lb) = local_batch.take() {
                if lb.count > 0 {
                    s.active_workers += 1;
                    return Task::Batch(lb);
                } else {
                    s.free_batches.push(lb);
                }
            }
            if s.active_workers == 0 {
                s.done = true;
                self.cond.notify_all();
                return Task::Done;
            }
            s = self.cond.wait(s).unwrap();
            if s.done && s.ready_batches.is_empty() && s.dirs_spans.is_empty() {
                return Task::Done;
            }
        }
    }

    fn finish_dir(
        &self,
        subdirs_arena: &[u8],
        subdirs_spans: &[(u32, u32)],
        local_batch: &mut Option<Box<Batch>>,
    ) {
        let mut s = self.mutex.lock().unwrap();
        debug_assert!(s.active_workers >= 1);
        let subdirs_len = subdirs_spans.len();
        if subdirs_len > 0 {
            let base = s.dirs_arena.len() as u32;
            s.dirs_arena.extend_from_slice(subdirs_arena);
            s.dirs_spans.reserve(subdirs_len);
            for &(off, len) in subdirs_spans {
                debug_assert!((off + len) as usize <= subdirs_arena.len());
                s.dirs_spans.push((base + off, len));
            }
        }

        let mut flushed = false;
        if let Some(lb) = local_batch.as_ref()
            && lb.count > 0
            && (lb.count >= 16 || (s.dirs_spans.is_empty() && s.active_workers == 1))
        {
            let flush_batch = local_batch.take().unwrap();
            s.ready_batches.push(flush_batch);
            flushed = true;
        }

        s.active_workers -= 1;

        let new_work = subdirs_len + usize::from(flushed);
        if new_work > 1 {
            self.cond.notify_all();
        } else if new_work == 1 {
            self.cond.notify_one();
        } else if s.active_workers == 0 && s.dirs_spans.is_empty() && s.ready_batches.is_empty() {
            s.done = true;
            self.cond.notify_all();
        }
    }
}

#[derive(Default, Clone, Copy)]
struct WorkerResult {
    lang_stats: LangStats,
    files_found: [i64; LANG_COUNT],
    files_processed: [i64; LANG_COUNT],
}

impl WorkerResult {
    fn add(&mut self, other: &Self) {
        self.lang_stats.add(&other.lang_stats);
        for i in 0..LANG_COUNT {
            self.files_found[i] += other.files_found[i];
            self.files_processed[i] += other.files_processed[i];
        }
    }

    fn total_files_found(&self) -> i64 {
        self.files_found.iter().sum()
    }
}

#[inline(always)]
fn read_file_into<Fd: AsFd>(dirfd: Fd, path: &CStr, fb: &mut Vec<u8>) -> bool {
    let Ok(fd) = openat(
        dirfd,
        path,
        OFlags::RDONLY | OFlags::NONBLOCK | OFlags::CLOEXEC,
        Mode::empty(),
    ) else {
        return false;
    };

    fb.clear();
    if fb.capacity() < 131072 {
        fb.reserve(131072 - fb.capacity());
    }

    loop {
        if fb.len() == fb.capacity() {
            fb.reserve(fb.capacity());
        }
        match read(&fd, spare_capacity(fb)) {
            Ok(0) => return true,
            Ok(_) => {
                if fb.len() < fb.capacity() {
                    return true;
                }
            }
            Err(Errno::INTR) => continue,
            Err(_) => {
                fb.clear();
                return false;
            }
        }
    }
}

#[inline(always)]
fn process_single_path<Fd: AsFd>(
    dirfd: Fd,
    path: &CStr,
    lang: LangId,
    res: &mut WorkerResult,
    fb: &mut Vec<u8>,
) {
    let idx = lang as usize;
    res.files_found[idx] += 1;
    if read_file_into(dirfd, path, fb) {
        res.files_processed[idx] += 1;
        parse_into(lang, fb, &mut res.lang_stats);
        if fb.capacity() > 524_288 {
            fb.clear();
            fb.shrink_to(131_072);
        }
    }
}

fn unified_worker_proc(sched: &WorkScheduler) -> WorkerResult {
    let enabled_mask = sched.enabled_mask;
    let mut res = WorkerResult::default();
    let mut fb = Vec::with_capacity(131072);
    let mut dents_buf: Vec<u8> = Vec::with_capacity(32768);
    let mut dir_buf: Vec<u8> = Vec::with_capacity(512);
    let mut subdirs_arena: Vec<u8> = Vec::with_capacity(4096);
    let mut subdirs_spans: Vec<(u32, u32)> = Vec::with_capacity(64);

    let mut local_batch: Option<Box<Batch>> = None;
    let mut prev_finished_batch: Option<Box<Batch>> = None;

    loop {
        let task = sched.complete_batch_and_next(
            prev_finished_batch.take(),
            &mut local_batch,
            &mut dir_buf,
        );

        match task {
            Task::Done => break,
            Task::Batch(mut b) => {
                if let Some(dirfd) = b.dirfd.take() {
                    for i in 0..b.count {
                        let name = b.get_rel_name_cstr(i);
                        process_single_path(&dirfd, name, b.langs[i], &mut res, &mut fb);
                    }
                } else {
                    for i in 0..b.count {
                        let path = b.get_path_cstr(i);
                        process_single_path(CWD, path, b.langs[i], &mut res, &mut fb);
                    }
                }
                prev_finished_batch = Some(b);
            }
            Task::Dir => {
                if local_batch.is_none() {
                    local_batch = Some(sched.acquire_batch());
                }
                let mut batch_started_empty = local_batch.as_ref().unwrap().count == 0;

                let dir_cstr = CStr::from_bytes_with_nul(&dir_buf).unwrap();
                let dir_bytes = &dir_buf[..dir_buf.len() - 1];

                if let Ok(fd) = openat(
                    CWD,
                    dir_cstr,
                    OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC,
                    Mode::empty(),
                ) {
                    subdirs_arena.clear();
                    subdirs_spans.clear();

                    let mut raw_dir = RawDir::new(&fd, dents_buf.spare_capacity_mut());
                    while let Some(Ok(entry)) = raw_dir.next() {
                        let mut ft = entry.file_type();
                        let name_cstr = entry.file_name();
                        let name = name_cstr.to_bytes();

                        if name == b"." || name == b".." {
                            continue;
                        }

                        if ft == FileType::Unknown
                            && let Ok(stx) = statx(
                                &fd,
                                name_cstr,
                                AtFlags::SYMLINK_NOFOLLOW,
                                StatxFlags::TYPE,
                            )
                        {
                            ft = FileType::from_raw_mode(RawMode::from(stx.stx_mode));
                        }

                        if ft == FileType::Directory {
                            let has_slash =
                                !dir_bytes.is_empty() && dir_bytes[dir_bytes.len() - 1] == b'/';
                            let sep_len = usize::from(!has_slash);
                            let sub_len = dir_bytes.len() + sep_len + name.len();
                            let start_off = subdirs_arena.len() as u32;
                            subdirs_arena.reserve(sub_len);
                            subdirs_arena.extend_from_slice(dir_bytes);
                            if !has_slash {
                                subdirs_arena.push(b'/');
                            }
                            subdirs_arena.extend_from_slice(name);
                            subdirs_spans.push((start_off, sub_len as u32));
                        } else if ft == FileType::RegularFile
                            && let Some(lang) = detect_language(name, enabled_mask)
                        {
                            let mut too_long = false;
                            let lb = local_batch.as_mut().unwrap();
                            if !lb.append(dir_bytes, name, lang, &mut too_long) {
                                if too_long {
                                    process_single_path(&fd, name_cstr, lang, &mut res, &mut fb);
                                } else {
                                    if batch_started_empty && let Ok(dup_fd) = dup(&fd) {
                                        lb.dirfd = Some(dup_fd);
                                    }
                                    let full = local_batch.take().unwrap();
                                    let mut next = sched.submit_batch_and_swap(full);
                                    batch_started_empty = true;
                                    let mut dummy = false;
                                    let ok = next.append(dir_bytes, name, lang, &mut dummy);
                                    assert!(ok);
                                    local_batch = Some(next);
                                }
                            }
                        }
                    }

                    if batch_started_empty
                        && let Some(lb) = local_batch.as_mut()
                        && lb.count >= 16
                        && let Ok(dup_fd) = dup(&fd)
                    {
                        lb.dirfd = Some(dup_fd);
                    }
                    drop(fd);
                    sched.finish_dir(&subdirs_arena, &subdirs_spans, &mut local_batch);
                } else {
                    sched.finish_dir(&[], &[], &mut local_batch);
                }
            }
        }
    }

    res
}

fn format_number(num: i64) -> String {
    let s = num.to_string();
    let bytes = s.as_bytes();
    let n = bytes.len();
    if num < 0 || n <= 3 {
        return s;
    }
    let commas = (n - 1) / 3;
    let total_len = n + commas;
    let mut out = vec![0u8; total_len];
    let mut dest = total_len;
    let mut src = n;
    let mut count = 0;
    while src > 0 {
        if count == 3 {
            dest -= 1;
            out[dest] = b',';
            count = 0;
        }
        dest -= 1;
        src -= 1;
        out[dest] = bytes[src];
        count += 1;
    }
    String::from_utf8(out).unwrap()
}

struct KeywordStat {
    keyword_id: u16,
    count: i64,
    percentage: f64,
}

fn should_color() -> bool {
    if env::var_os("NO_COLOR").is_some() {
        return false;
    }
    rustix::termios::isatty(stdout())
}

fn print_detailed_results(
    totals: &WorkerResult,
    lang: LangId,
    processing_time: f64,
    num_workers: usize,
    cpu_count: usize,
) {
    let color = should_color();
    let (c_bold, c_cyan, c_gray, c_reset) = if color {
        ("\x1b[1m", "\x1b[36m", "\x1b[90m", "\x1b[0m")
    } else {
        ("", "", "", "")
    };

    let idx = lang as usize;
    let files_found = totals.files_found[idx];
    let files_processed = totals.files_processed[idx];
    let lines = totals.lang_stats.lines(lang);
    let code = totals.lang_stats.code_lines(lang);
    let comments = totals.lang_stats.comment_lines(lang);
    let blanks = totals.lang_stats.blank_lines(lang);
    let total_kw = totals.lang_stats.total_keywords(lang);
    let kw_names = LangStats::keyword_names(lang);
    let kw_counts = totals.lang_stats.keyword_counts(lang);

    let mut out = String::with_capacity(8192);
    let _ = writeln!(
        out,
        "{}Processed {} files with {} worker threads (CPU cores: {})...{}",
        c_gray,
        format_number(files_found),
        num_workers,
        cpu_count,
        c_reset
    );
    let _ = writeln!(
        out,
        "{}{}{} Keyword Analyzer (Unified Work-Stealing)",
        c_bold,
        lang.name(),
        c_reset
    );
    let _ = writeln!(out, "Files processed: {}", format_number(files_processed));
    let _ = writeln!(
        out,
        "Lines: {} (Code: {}, Comments: {}, Blanks: {})",
        format_number(lines),
        format_number(code),
        format_number(comments),
        format_number(blanks)
    );
    let _ = writeln!(out, "Total keywords: {}", format_number(total_kw));
    let _ = writeln!(out, "Processing time: {:.3} seconds", processing_time);

    let mut stats = Vec::with_capacity(kw_names.len());
    for (i, &c) in kw_counts.iter().enumerate() {
        if c > 0 {
            let pct = if total_kw > 0 {
                (c as f64 / total_kw as f64) * 100.0
            } else {
                0.0
            };
            stats.push(KeywordStat {
                keyword_id: i as u16,
                count: c,
                percentage: pct,
            });
        }
    }

    stats.sort_unstable_by(|a, b| match b.count.cmp(&a.count) {
        Ordering::Equal => kw_names[a.keyword_id as usize].cmp(kw_names[b.keyword_id as usize]),
        ord => ord,
    });

    let _ = writeln!(out, "\n{}Keyword frequencies (most frequent first):{}", c_bold, c_reset);
    let _ = writeln!(
        out,
        "{}{:>5}  {:<16} {:>10}  {:>7}{}",
        c_cyan, "Rank", "Keyword", "Count", "Percent", c_reset
    );
    let _ = writeln!(
        out,
        "{}─────  ──────────────── ──────────  ───────{}",
        c_gray, c_reset
    );

    for (i, st) in stats.iter().enumerate() {
        let _ = writeln!(
            out,
            "{:>5}  {:<16} {:>10}  {:6.2}%",
            i + 1,
            kw_names[st.keyword_id as usize],
            format_number(st.count),
            st.percentage
        );
    }

    let _ = write(stdout(), out.as_bytes());
}

fn print_summary_results(
    totals: &WorkerResult,
    processing_time: f64,
    num_workers: usize,
    cpu_count: usize,
) {
    let color = should_color();
    let (c_cyan, c_white, c_green, c_gray, c_reset) = if color {
        (
            "\x1b[36m",
            "\x1b[1;37m",
            "\x1b[1;32m",
            "\x1b[90m",
            "\x1b[0m",
        )
    } else {
        ("", "", "", "", "")
    };

    let mut out = String::with_capacity(4096);
    let border_top = "┌────────────────────┬──────────┬──────────────┬──────────────┬──────────────┬──────────────┬──────────────┐";
    let border_sep = "├────────────────────┼──────────┼──────────────┼──────────────┼──────────────┼──────────────┼──────────────┤";
    let border_bot = "└────────────────────┴──────────┴──────────────┴──────────────┴──────────────┴──────────────┴──────────────┘";

    let _ = writeln!(out, "{}{}{}", c_gray, border_top, c_reset);
    let _ = writeln!(
        out,
        "{}│{} {:<18} {}│{} {:>8} {}│{} {:>12} {}│{} {:>12} {}│{} {:>12} {}│{} {:>12} {}│{} {:>12} {}│",
        c_gray, c_white, "Language",
        c_gray, c_white, "Files",
        c_gray, c_white, "Lines",
        c_gray, c_white, "Code",
        c_gray, c_white, "Comments",
        c_gray, c_white, "Blanks",
        c_gray, c_white, "Keywords",
        c_gray
    );
    let _ = writeln!(out, "{}{}{}", c_gray, border_sep, c_reset);

    let mut rows: Vec<(LangId, i64, i64, i64, i64, i64, i64)> = Vec::with_capacity(LANG_COUNT);
    let mut total_files = 0i64;
    let mut total_lines = 0i64;
    let mut total_code = 0i64;
    let mut total_comments = 0i64;
    let mut total_blanks = 0i64;
    let mut total_keywords = 0i64;

    for lang in ALL_LANGS {
        let idx = lang as usize;
        let files = totals.files_processed[idx];
        if files > 0 {
            let lines = totals.lang_stats.lines(lang);
            let code = totals.lang_stats.code_lines(lang);
            let comments = totals.lang_stats.comment_lines(lang);
            let blanks = totals.lang_stats.blank_lines(lang);
            let kws = totals.lang_stats.total_keywords(lang);
            total_files += files;
            total_lines += lines;
            total_code += code;
            total_comments += comments;
            total_blanks += blanks;
            total_keywords += kws;
            rows.push((lang, files, lines, code, comments, blanks, kws));
        }
    }

    rows.sort_unstable_by(|a, b| b.6.cmp(&a.6).then_with(|| a.0.name().cmp(b.0.name())));

    for (lang, files, lines, code, comments, blanks, kws) in rows {
        let _ = writeln!(
            out,
            "{}│{} {:<18} {}│{} {:>8} {}│{} {:>12} {}│{} {:>12} {}│{} {:>12} {}│{} {:>12} {}│{} {:>12} {}│",
            c_gray, c_cyan, lang.name(),
            c_gray, c_reset, format_number(files),
            c_gray, c_reset, format_number(lines),
            c_gray, c_reset, format_number(code),
            c_gray, c_reset, format_number(comments),
            c_gray, c_reset, format_number(blanks),
            c_gray, c_reset, format_number(kws),
            c_gray
        );
    }

    let _ = writeln!(out, "{}{}{}", c_gray, border_sep, c_reset);
    let _ = writeln!(
        out,
        "{}│{} {:<18} {}│{} {:>8} {}│{} {:>12} {}│{} {:>12} {}│{} {:>12} {}│{} {:>12} {}│{} {:>12} {}│",
        c_gray, c_green, "Total",
        c_gray, c_white, format_number(total_files),
        c_gray, c_white, format_number(total_lines),
        c_gray, c_white, format_number(total_code),
        c_gray, c_white, format_number(total_comments),
        c_gray, c_white, format_number(total_blanks),
        c_gray, c_white, format_number(total_keywords),
        c_gray
    );
    let _ = writeln!(out, "{}{}{}", c_gray, border_bot, c_reset);
    let _ = writeln!(
        out,
        "{}Processed {} files with {} worker threads (CPU cores: {}) in {:.3}s{}",
        c_gray,
        format_number(totals.total_files_found()),
        num_workers,
        cpu_count,
        processing_time,
        c_reset
    );

    let _ = write(stdout(), out.as_bytes());
}

fn get_cpu_count() -> usize {
    thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
}

fn print_usage(prog: &str) -> ! {
    println!(
        "Usage: {} [FLAGS] <file_or_directory>\nAnalyzes source code keyword frequencies\n\nFlags:\n  -c          Show detailed C keyword frequencies\n  -cpp        Show detailed C++ keyword frequencies\n  -rs, -rust  Show detailed Rust keyword frequencies\n  -py, -python Show detailed Python keyword frequencies\n  -pl, -perl  Show detailed Perl keyword frequencies\n  -go, -golang Show detailed Go keyword frequencies\n  -zig        Show detailed Zig keyword frequencies\n  -odin       Show detailed Odin keyword frequencies\n  -js, -javascript Show detailed JavaScript keyword frequencies\n  -ts, -typescript Show detailed TypeScript keyword frequencies",
        prog
    );
    process::exit(1);
}

fn parse_cli_args<'a>(args: &'a [std::ffi::OsString]) -> (&'a OsStr, u32, Vec<LangId>) {
    let prog = args
        .first()
        .map(|s| s.to_string_lossy())
        .unwrap_or_else(|| "lfa".into());

    if args.len() < 2 {
        print_usage(&prog);
    }

    let mut target_os: Option<&'a OsStr> = None;
    let mut flag_mask = 0u32;
    let mut selected_langs: Vec<LangId> = Vec::new();

    for arg in &args[1..] {
        let bytes = arg.as_bytes();
        if bytes.starts_with(b"-") && bytes.len() > 1 {
            if let Some(lang) = LangId::from_flag(bytes) {
                flag_mask |= lang.mask();
                if !selected_langs.contains(&lang) {
                    selected_langs.push(lang);
                }
            } else {
                eprintln!("Error: Unknown flag {}", arg.to_string_lossy());
                print_usage(&prog);
            }
        } else if target_os.is_none() {
            target_os = Some(arg.as_os_str());
        } else {
            eprintln!("Error: Unexpected argument {}", arg.to_string_lossy());
            print_usage(&prog);
        }
    }

    let Some(target) = target_os else {
        print_usage(&prog);
    };

    let enabled_mask = if flag_mask != 0 {
        flag_mask
    } else {
        LANG_ALL_MASK
    };

    (target, enabled_mask, selected_langs)
}

fn main() {
    let start = Instant::now();
    let args: Vec<_> = env::args_os().collect();
    let (target_os, enabled_mask, selected_langs) = parse_cli_args(&args);
    let target_bytes = target_os.as_bytes();

    let Ok(stx) = statx(CWD, target_bytes, AtFlags::empty(), StatxFlags::TYPE) else {
        eprintln!("Error: Cannot access {}", target_os.to_string_lossy());
        process::exit(1);
    };

    let ft = FileType::from_raw_mode(RawMode::from(stx.stx_mode));
    let cpu_count = get_cpu_count();
    let mut totals = WorkerResult::default();
    let mut num_workers = cpu_count;

    if ft == FileType::Directory {
        let sched = WorkScheduler::new(target_bytes, enabled_mask);
        let extra_threads = cpu_count.saturating_sub(1);

        thread::scope(|s| {
            let mut handles = Vec::with_capacity(extra_threads);
            for _ in 0..extra_threads {
                let sched_ref = &sched;
                handles.push(
                    thread::Builder::new()
                        .stack_size(512 * 1024)
                        .spawn_scoped(s, move || unified_worker_proc(sched_ref))
                        .unwrap(),
                );
            }
            let main_res = unified_worker_proc(&sched);
            totals.add(&main_res);
            for h in handles {
                totals.add(&h.join().unwrap());
            }
        });
    } else {
        let Some(lang) = (if ft == FileType::RegularFile {
            detect_language(target_bytes, enabled_mask)
        } else {
            None
        }) else {
            eprintln!(
                "Error: {} is not a supported source file",
                target_os.to_string_lossy()
            );
            process::exit(1);
        };
        num_workers = 1;
        let idx = lang as usize;
        totals.files_found[idx] = 1;
        let mut target_nul = Vec::with_capacity(target_bytes.len() + 1);
        target_nul.extend_from_slice(target_bytes);
        target_nul.push(0);
        if let Ok(target_cstr) = CStr::from_bytes_with_nul(&target_nul) {
            let mut fb = Vec::with_capacity(131072);
            if read_file_into(CWD, target_cstr, &mut fb) {
                totals.files_processed[idx] = 1;
                parse_into(lang, &fb, &mut totals.lang_stats);
            }
        }
    }

    if totals.total_files_found() == 0 {
        println!("No supported source files found");
        process::exit(1);
    }

    let processing_time = start.elapsed().as_secs_f64();
    if selected_langs.is_empty() {
        print_summary_results(&totals, processing_time, num_workers, cpu_count);
    } else {
        for (i, &lang) in selected_langs.iter().enumerate() {
            if i > 0 {
                println!();
            }
            print_detailed_results(&totals, lang, processing_time, num_workers, cpu_count);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustix::fs::{mkdirat, mknodat, symlinkat, unlinkat};
    use std::ffi::OsString;
    use std::fs;

    fn find_c_kw_id(name: &str) -> usize {
        lang::c::KEYWORD_NAMES
            .iter()
            .position(|&kw| kw == name)
            .unwrap()
    }

    fn find_cpp_kw_id(name: &str) -> usize {
        lang::cpp::KEYWORD_NAMES
            .iter()
            .position(|&kw| kw == name)
            .unwrap()
    }

    #[test]
    fn test_directory_traversal_and_edge_cases() {
        let tmp_dir = format!("/tmp/lfa_rust_test_{}", process::id());
        let _ = fs::remove_dir_all(&tmp_dir);
        mkdirat(CWD, tmp_dir.as_bytes(), Mode::RWXU).unwrap();

        let path_c = format!("{}/valid.c", tmp_dir);
        let path_h = format!("{}/UPPER.H", tmp_dir);
        let path_cpp = format!("{}/module.cpp", tmp_dir);
        let path_ignored = format!("{}/ignored.txt", tmp_dir);
        let path_fifo = format!("{}/named_pipe.c", tmp_dir);
        let sub_dir = format!("{}/subdir", tmp_dir);
        let sub_c = format!("{}/subdir/nested.c", tmp_dir);

        fs::write(&path_c, "int main(void) { if (true) return 0; }\n").unwrap();
        fs::write(&path_h, "typedef struct { constexpr int x; } Foo;\n").unwrap();
        fs::write(
            &path_cpp,
            "namespace n { class C final { public: void f() override {} }; }\n",
        )
        .unwrap();
        fs::write(&path_ignored, "int while for return if switch case default;\n").unwrap();
        mknodat(
            CWD,
            path_fifo.as_bytes(),
            FileType::Fifo,
            Mode::RUSR | Mode::WUSR,
            0,
        )
        .unwrap();
        mkdirat(CWD, sub_dir.as_bytes(), Mode::RWXU).unwrap();
        fs::write(
            &sub_c,
            "static inline _Bool check(void) { return false; }\n",
        )
        .unwrap();

        let sched = WorkScheduler::new(tmp_dir.as_bytes(), LANG_ALL_MASK);
        let mut totals = WorkerResult::default();

        thread::scope(|s| {
            let mut handles = Vec::new();
            for _ in 0..3 {
                let sched_ref = &sched;
                handles.push(s.spawn(move || unified_worker_proc(sched_ref)));
            }
            let main_res = unified_worker_proc(&sched);
            totals.add(&main_res);
            for h in handles {
                totals.add(&h.join().unwrap());
            }
        });

        let c_idx = LangId::C as usize;
        let cpp_idx = LangId::Cpp as usize;
        assert_eq!(totals.files_found[c_idx], 3);
        assert_eq!(totals.files_processed[c_idx], 3);
        assert_eq!(totals.lang_stats.c.lines_processed, 3);
        assert_eq!(totals.lang_stats.c.total_keywords, 15);
        assert_eq!(totals.lang_stats.c.keyword_counts[find_c_kw_id("int")], 2);
        assert_eq!(totals.lang_stats.c.keyword_counts[find_c_kw_id("void")], 2);
        assert_eq!(
            totals.lang_stats.c.keyword_counts[find_c_kw_id("return")],
            2
        );
        assert_eq!(totals.lang_stats.c.keyword_counts[find_c_kw_id("true")], 1);
        assert_eq!(totals.lang_stats.c.keyword_counts[find_c_kw_id("false")], 1);
        assert_eq!(
            totals.lang_stats.c.keyword_counts[find_c_kw_id("constexpr")],
            1
        );

        assert_eq!(totals.files_found[cpp_idx], 1);
        assert_eq!(totals.files_processed[cpp_idx], 1);
        assert_eq!(totals.lang_stats.cpp.lines_processed, 1);
        assert_eq!(totals.lang_stats.cpp.total_keywords, 6);
        assert_eq!(
            totals.lang_stats.cpp.keyword_counts[find_cpp_kw_id("namespace")],
            1
        );
        assert_eq!(
            totals.lang_stats.cpp.keyword_counts[find_cpp_kw_id("class")],
            1
        );
        assert_eq!(
            totals.lang_stats.cpp.keyword_counts[find_cpp_kw_id("final")],
            1
        );
        assert_eq!(
            totals.lang_stats.cpp.keyword_counts[find_cpp_kw_id("public")],
            1
        );
        assert_eq!(
            totals.lang_stats.cpp.keyword_counts[find_cpp_kw_id("void")],
            1
        );
        assert_eq!(
            totals.lang_stats.cpp.keyword_counts[find_cpp_kw_id("override")],
            1
        );

        let c_only_sched = WorkScheduler::new(tmp_dir.as_bytes(), LangId::C.mask());
        let c_only_res = unified_worker_proc(&c_only_sched);
        assert_eq!(c_only_res.files_found[c_idx], 3);
        assert_eq!(c_only_res.files_found[cpp_idx], 0);

        let cpp_only_sched = WorkScheduler::new(tmp_dir.as_bytes(), LangId::Cpp.mask());
        let cpp_only_res = unified_worker_proc(&cpp_only_sched);
        assert_eq!(cpp_only_res.files_found[c_idx], 0);
        assert_eq!(cpp_only_res.files_found[cpp_idx], 2);

        unlinkat(CWD, sub_c.as_bytes(), AtFlags::empty()).unwrap();
        unlinkat(CWD, sub_dir.as_bytes(), AtFlags::REMOVEDIR).unwrap();
        unlinkat(CWD, path_fifo.as_bytes(), AtFlags::empty()).unwrap();
        unlinkat(CWD, path_ignored.as_bytes(), AtFlags::empty()).unwrap();
        unlinkat(CWD, path_cpp.as_bytes(), AtFlags::empty()).unwrap();
        unlinkat(CWD, path_h.as_bytes(), AtFlags::empty()).unwrap();
        unlinkat(CWD, path_c.as_bytes(), AtFlags::empty()).unwrap();
        unlinkat(CWD, tmp_dir.as_bytes(), AtFlags::REMOVEDIR).unwrap();
    }

    #[test]
    fn test_batch_limits_and_slash_normalization() {
        let mut batch = Batch::new();
        let mut too_long = false;

        assert!(batch.append(b"/tmp/dir", b"a.c", LangId::C, &mut too_long));
        assert!(!too_long);
        assert_eq!(batch.get_path_cstr(0).to_bytes(), b"/tmp/dir/a.c");
        assert_eq!(batch.get_rel_name_cstr(0).to_bytes(), b"a.c");

        assert!(batch.append(b"/tmp/dir/", b"b.h", LangId::C, &mut too_long));
        assert!(!too_long);
        assert_eq!(batch.get_path_cstr(1).to_bytes(), b"/tmp/dir/b.h");
        assert_eq!(batch.get_rel_name_cstr(1).to_bytes(), b"b.h");

        let huge_dir = vec![b'x'; BATCH_BUF_SIZE];
        assert!(!batch.append(&huge_dir, b"c.c", LangId::C, &mut too_long));
        assert!(too_long);
        assert_eq!(batch.count, 2);

        batch.reset();
        for i in 0..BATCH_SIZE {
            let name = format!("f{}.c", i);
            assert!(batch.append(b"/d", name.as_bytes(), LangId::C, &mut too_long));
            assert!(!too_long);
        }
        assert_eq!(batch.count, BATCH_SIZE);
        assert!(!batch.append(b"/d", b"overflow.c", LangId::C, &mut too_long));
        assert!(!too_long);
    }

    #[test]
    fn test_large_directory_multi_batch_and_symlinks() {
        let tmp_dir = format!("/tmp/lfa_multi_batch_test_{}/", process::id());
        let _ = fs::remove_dir_all(&tmp_dir);
        mkdirat(CWD, tmp_dir.as_bytes(), Mode::RWXU).unwrap();

        let file_count = 150usize;
        for i in 0..file_count {
            let p = format!("{}file_{}.c", tmp_dir, i);
            fs::write(&p, "int x = 0;\nreturn x;\n").unwrap();
        }

        let empty_c = format!("{}empty.c", tmp_dir);
        fs::write(&empty_c, "").unwrap();

        let broken_link = format!("{}broken.c", tmp_dir);
        symlinkat(&b"nonexistent_target.c"[..], CWD, broken_link.as_bytes()).unwrap();

        let sched = WorkScheduler::new(tmp_dir.as_bytes(), LANG_ALL_MASK);
        let mut totals = WorkerResult::default();

        thread::scope(|s| {
            let mut handles = Vec::new();
            for _ in 0..4 {
                let sched_ref = &sched;
                handles.push(s.spawn(move || unified_worker_proc(sched_ref)));
            }
            let main_res = unified_worker_proc(&sched);
            totals.add(&main_res);
            for h in handles {
                totals.add(&h.join().unwrap());
            }
        });

        let c_idx = LangId::C as usize;
        assert_eq!(totals.files_found[c_idx], (file_count + 1) as i64);
        assert_eq!(totals.files_processed[c_idx], (file_count + 1) as i64);
        assert_eq!(totals.lang_stats.c.lines_processed, (file_count * 2) as i64);
        assert_eq!(totals.lang_stats.c.total_keywords, (file_count * 2) as i64);

        let disabled_sched = WorkScheduler::new(tmp_dir.as_bytes(), 0);
        let disabled_res = unified_worker_proc(&disabled_sched);
        assert_eq!(disabled_res.total_files_found(), 0);

        let _ = fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn test_cli_flag_parsing() {
        let args = vec![OsString::from("lfa"), OsString::from("/tmp")];
        let (target, mask, detailed) = parse_cli_args(&args);
        assert_eq!(target.as_bytes(), b"/tmp");
        assert_eq!(mask, LANG_ALL_MASK);
        assert_eq!(detailed, Vec::<LangId>::new());

        let args_c = vec![
            OsString::from("lfa"),
            OsString::from("/tmp"),
            OsString::from("-c"),
        ];
        let (target, mask, detailed) = parse_cli_args(&args_c);
        assert_eq!(target.as_bytes(), b"/tmp");
        assert_eq!(mask, LangId::C.mask());
        assert_eq!(detailed, vec![LangId::C]);

        let args_c_first = vec![
            OsString::from("lfa"),
            OsString::from("-c"),
            OsString::from("/tmp"),
        ];
        let (target, mask, detailed) = parse_cli_args(&args_c_first);
        assert_eq!(target.as_bytes(), b"/tmp");
        assert_eq!(mask, LangId::C.mask());
        assert_eq!(detailed, vec![LangId::C]);

        let args_cpp = vec![
            OsString::from("lfa"),
            OsString::from("-cpp"),
            OsString::from("/tmp"),
        ];
        let (target, mask, detailed) = parse_cli_args(&args_cpp);
        assert_eq!(target.as_bytes(), b"/tmp");
        assert_eq!(mask, LangId::Cpp.mask());
        assert_eq!(detailed, vec![LangId::Cpp]);

        for flag in ["-rs", "--rs", "-rust", "--rust"] {
            let args_rs = vec![
                OsString::from("lfa"),
                OsString::from(flag),
                OsString::from("/tmp"),
            ];
            let (target, mask, detailed) = parse_cli_args(&args_rs);
            assert_eq!(target.as_bytes(), b"/tmp");
            assert_eq!(mask, LangId::Rust.mask());
            assert_eq!(detailed, vec![LangId::Rust]);
        }

        for flag in ["-py", "--py", "-python", "--python"] {
            let args_py = vec![
                OsString::from("lfa"),
                OsString::from(flag),
                OsString::from("/tmp"),
            ];
            let (target, mask, detailed) = parse_cli_args(&args_py);
            assert_eq!(target.as_bytes(), b"/tmp");
            assert_eq!(mask, LangId::Python.mask());
            assert_eq!(detailed, vec![LangId::Python]);
        }

        for flag in ["-pl", "--pl", "-perl", "--perl"] {
            let args_pl = vec![
                OsString::from("lfa"),
                OsString::from(flag),
                OsString::from("/tmp"),
            ];
            let (target, mask, detailed) = parse_cli_args(&args_pl);
            assert_eq!(target.as_bytes(), b"/tmp");
            assert_eq!(mask, LangId::Perl.mask());
            assert_eq!(detailed, vec![LangId::Perl]);
        }

        for flag in ["-go", "--go", "-golang", "--golang"] {
            let args_go = vec![
                OsString::from("lfa"),
                OsString::from(flag),
                OsString::from("/tmp"),
            ];
            let (target, mask, detailed) = parse_cli_args(&args_go);
            assert_eq!(target.as_bytes(), b"/tmp");
            assert_eq!(mask, LangId::Go.mask());
            assert_eq!(detailed, vec![LangId::Go]);
        }

        for flag in ["-zig", "--zig"] {
            let args_zig = vec![
                OsString::from("lfa"),
                OsString::from(flag),
                OsString::from("/tmp"),
            ];
            let (target, mask, detailed) = parse_cli_args(&args_zig);
            assert_eq!(target.as_bytes(), b"/tmp");
            assert_eq!(mask, LangId::Zig.mask());
            assert_eq!(detailed, vec![LangId::Zig]);
        }

        for flag in ["-odin", "--odin"] {
            let args_odin = vec![
                OsString::from("lfa"),
                OsString::from(flag),
                OsString::from("/tmp"),
            ];
            let (target, mask, detailed) = parse_cli_args(&args_odin);
            assert_eq!(target.as_bytes(), b"/tmp");
            assert_eq!(mask, LangId::Odin.mask());
            assert_eq!(detailed, vec![LangId::Odin]);
        }

        for flag in ["-js", "--js", "-javascript", "--javascript"] {
            let args_js = vec![
                OsString::from("lfa"),
                OsString::from(flag),
                OsString::from("/tmp"),
            ];
            let (target, mask, detailed) = parse_cli_args(&args_js);
            assert_eq!(target.as_bytes(), b"/tmp");
            assert_eq!(mask, LangId::JavaScript.mask());
            assert_eq!(detailed, vec![LangId::JavaScript]);
        }

        for flag in ["-ts", "--ts", "-typescript", "--typescript"] {
            let args_ts = vec![
                OsString::from("lfa"),
                OsString::from(flag),
                OsString::from("/tmp"),
            ];
            let (target, mask, detailed) = parse_cli_args(&args_ts);
            assert_eq!(target.as_bytes(), b"/tmp");
            assert_eq!(mask, LangId::TypeScript.mask());
            assert_eq!(detailed, vec![LangId::TypeScript]);
        }

        let args_multi = vec![
            OsString::from("lfa"),
            OsString::from("-zig"),
            OsString::from("/tmp"),
            OsString::from("-c"),
            OsString::from("-zig"),
            OsString::from("-odin"),
        ];
        let (target, mask, detailed) = parse_cli_args(&args_multi);
        assert_eq!(target.as_bytes(), b"/tmp");
        assert_eq!(
            mask,
            LangId::Zig.mask() | LangId::C.mask() | LangId::Odin.mask()
        );
        assert_eq!(detailed, vec![LangId::Zig, LangId::C, LangId::Odin]);
    }

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(0), "0");
        assert_eq!(format_number(-42), "-42");
        assert_eq!(format_number(999), "999");
        assert_eq!(format_number(1000), "1,000");
        assert_eq!(format_number(1234567), "1,234,567");
        assert_eq!(format_number(10000000000), "10,000,000,000");
        assert!(get_cpu_count() >= 1);
    }
}
