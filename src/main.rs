#![forbid(unsafe_code)]

mod lang;

use lang::doc::{
    detect_doc_format, parse_doc_into, AllDocStats, DocFormatId, FileTarget, ALL_DOC_FORMATS,
    DOC_FORMAT_COUNT,
};
#[cfg(test)]
use lang::doc::DocStats;
use lang::{
    detect_language, detect_shebang, parse_into, LangId, LangStats, ALL_LANGS, LANG_ALL_MASK,
    LANG_COUNT, LANG_OBJECTIVEC_MASK,
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
use std::io::Write as _;
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
    targets: [FileTarget; BATCH_SIZE],
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
            targets: [FileTarget::from_lang(LangId::C as u8); BATCH_SIZE],
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
    fn append(&mut self, dir: &[u8], name: &[u8], target: FileTarget, too_long: &mut bool) -> bool {
        assert!(!name.is_empty());
        assert!(self.count <= BATCH_SIZE);
        assert!(self.data_len <= BATCH_BUF_SIZE);
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
        self.targets[self.count] = target;
        self.data_len = off + needed;
        self.count += 1;
        assert!(self.count <= BATCH_SIZE);
        assert!(self.data_len <= BATCH_BUF_SIZE);
        true
    }

    #[inline(always)]
    fn get_path_cstr(&self, idx: usize) -> &CStr {
        assert!(idx < self.count);
        let off = self.offsets[idx] as usize;
        let len = self.lens[idx] as usize;
        assert!(off + len < self.data_len);
        assert!(self.data[off + len] == 0);
        CStr::from_bytes_with_nul(&self.data[off..off + len + 1]).unwrap()
    }

    #[inline(always)]
    fn get_rel_name_cstr(&self, idx: usize) -> &CStr {
        assert!(idx < self.count);
        let off = self.offsets[idx] as usize;
        let name_off = self.name_offsets[idx] as usize;
        let len = self.lens[idx] as usize;
        assert!(name_off >= off && off + len < self.data_len);
        assert!(self.data[off + len] == 0);
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
    enabled_mask: u128,
    hidden: bool,
    mutex: Mutex<SchedulerState>,
    cond: Condvar,
}

impl WorkScheduler {
    fn new(root: &[u8], enabled_mask: u128, hidden: bool) -> Self {
        assert!(!root.is_empty());
        let mut dirs_arena = Vec::with_capacity(65536);
        dirs_arena.extend_from_slice(root);
        let mut dirs_spans = Vec::with_capacity(256);
        dirs_spans.push((0, root.len() as u32));

        Self {
            enabled_mask,
            hidden,
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
        assert!(full_batch.count > 0 && full_batch.count <= BATCH_SIZE);
        assert!(full_batch.data_len <= BATCH_BUF_SIZE);
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
            assert!(s.active_workers >= 1);
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
                assert!(end <= s.dirs_arena.len());
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
        assert!(s.active_workers >= 1);
        let subdirs_len = subdirs_spans.len();
        if subdirs_len > 0 {
            let base = s.dirs_arena.len() as u32;
            s.dirs_arena.extend_from_slice(subdirs_arena);
            s.dirs_spans.reserve(subdirs_len);
            for &(off, len) in subdirs_spans {
                assert!((off + len) as usize <= subdirs_arena.len());
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

#[derive(Clone, Copy)]
struct WorkerResult {
    lang_stats: LangStats,
    doc_stats: AllDocStats,
    files_found: [i64; LANG_COUNT],
    files_processed: [i64; LANG_COUNT],
    doc_files_found: [i64; DOC_FORMAT_COUNT],
    doc_files_processed: [i64; DOC_FORMAT_COUNT],
}

impl Default for WorkerResult {
    fn default() -> Self {
        Self {
            lang_stats: LangStats::default(),
            doc_stats: AllDocStats::default(),
            files_found: [0; LANG_COUNT],
            files_processed: [0; LANG_COUNT],
            doc_files_found: [0; DOC_FORMAT_COUNT],
            doc_files_processed: [0; DOC_FORMAT_COUNT],
        }
    }
}

impl WorkerResult {
    fn add(&mut self, other: &Self) {
        self.lang_stats.add(&other.lang_stats);
        self.doc_stats.add(&other.doc_stats);
        for i in 0..LANG_COUNT {
            self.files_found[i] += other.files_found[i];
            self.files_processed[i] += other.files_processed[i];
            assert!(self.files_processed[i] <= self.files_found[i]);
        }
        for i in 0..DOC_FORMAT_COUNT {
            self.doc_files_found[i] += other.doc_files_found[i];
            self.doc_files_processed[i] += other.doc_files_processed[i];
            assert!(self.doc_files_processed[i] <= self.doc_files_found[i]);
        }
    }

    fn total_files_found(&self) -> i64 {
        let lang_total: i64 = self.files_found.iter().sum();
        let doc_total: i64 = self.doc_files_found.iter().sum();
        let total = lang_total + doc_total;
        assert!(total >= 0);
        total
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
    target: FileTarget,
    enabled_mask: u128,
    res: &mut WorkerResult,
    fb: &mut Vec<u8>,
) {
    if target.is_doc() {
        if let Some(doc) = target.doc_format() {
            let idx = doc as usize;
            res.doc_files_found[idx] += 1;
            if read_file_into(dirfd, path, fb) {
                res.doc_files_processed[idx] += 1;
                parse_doc_into(doc, fb, &mut res.doc_stats.stats[idx]);
                if fb.capacity() > 524_288 {
                    fb.clear();
                    fb.shrink_to(131_072);
                }
            }
        }
    } else {
        let mut lang = ALL_LANGS[target.lang_id() as usize];
        if read_file_into(dirfd, path, fb) {
            if lang == LangId::Matlab
                && (enabled_mask & LANG_OBJECTIVEC_MASK) != 0
                && lang::objc::is_objc_file(fb)
            {
                lang = LangId::ObjectiveC;
            }
            let idx = lang as usize;
            res.files_found[idx] += 1;
            res.files_processed[idx] += 1;
            parse_into(lang, fb, &mut res.lang_stats);
            if fb.capacity() > 524_288 {
                fb.clear();
                fb.shrink_to(131_072);
            }
        } else {
            res.files_found[lang as usize] += 1;
        }
    }
}

fn unified_worker_proc(sched: &WorkScheduler) -> WorkerResult {
    let enabled_mask = sched.enabled_mask;
    let hidden = sched.hidden;
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
                        process_single_path(&dirfd, name, b.targets[i], enabled_mask, &mut res, &mut fb);
                    }
                } else {
                    for i in 0..b.count {
                        let path = b.get_path_cstr(i);
                        process_single_path(CWD, path, b.targets[i], enabled_mask, &mut res, &mut fb);
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
                        if !hidden && name[0] == b'.' {
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
                        } else if ft == FileType::RegularFile {
                            let mut target_opt = detect_language(name, enabled_mask)
                                .map(|l| FileTarget::from_lang(l as u8))
                                .or_else(|| {
                                    if enabled_mask == LANG_ALL_MASK {
                                        detect_doc_format(name).map(FileTarget::from_doc)
                                    } else {
                                        None
                                    }
                                })
                                .or_else(|| {
                                    if !name.contains(&b'.') {
                                        let mut probe_buf = [0u8; 128];
                                        if let Ok(file_fd) = openat(&fd, name_cstr, OFlags::RDONLY | OFlags::CLOEXEC, Mode::empty())
                                            && let Ok(n) = read(&file_fd, &mut probe_buf)
                                        {
                                            return detect_shebang(&probe_buf[..n], enabled_mask)
                                                .map(|l| FileTarget::from_lang(l as u8));
                                        }
                                    }
                                    None
                                });

                            if let Some(target) = target_opt
                                && !target.is_doc()
                                && target.lang_id() == LangId::D as u8
                                && name.ends_with(b".d")
                            {
                                let mut probe_buf = [0u8; 128];
                                if let Ok(file_fd) = openat(&fd, name_cstr, OFlags::RDONLY | OFlags::CLOEXEC, Mode::empty())
                                    && let Ok(n) = read(&file_fd, &mut probe_buf)
                                    && lang::d::is_d_dep_file(&probe_buf[..n])
                                {
                                    target_opt = None;
                                }
                            }

                            if let Some(target) = target_opt {
                                let mut too_long = false;
                                let lb = local_batch.as_mut().unwrap();
                                if !lb.append(dir_bytes, name, target, &mut too_long) {
                                    if too_long {
                                        process_single_path(&fd, name_cstr, target, enabled_mask, &mut res, &mut fb);
                                    } else {
                                        if batch_started_empty && let Ok(dup_fd) = dup(&fd) {
                                            lb.dirfd = Some(dup_fd);
                                        }
                                        let full = local_batch.take().unwrap();
                                        let mut next = sched.submit_batch_and_swap(full);
                                        batch_started_empty = true;
                                        let mut dummy = false;
                                        let ok = next.append(dir_bytes, name, target, &mut dummy);
                                        assert!(ok);
                                        local_batch = Some(next);
                                    }
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
    assert_eq!(dest, 0);
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

    let m_border_top = "┌────────────────────┬──────────┬──────────────┬──────────────┬──────────────┬──────────────┬──────────────┐";
    let m_border_sep = "├────────────────────┼──────────┼──────────────┼──────────────┼──────────────┼──────────────┼──────────────┤";
    let m_border_bot = "└────────────────────┴──────────┴──────────────┴──────────────┴──────────────┴──────────────┴──────────────┘";

    let _ = writeln!(out, "{}{}{}", c_gray, m_border_top, c_reset);
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
    let _ = writeln!(out, "{}{}{}", c_gray, m_border_sep, c_reset);
    let _ = writeln!(
        out,
        "{}│{} {:<18} {}│{} {:>8} {}│{} {:>12} {}│{} {:>12} {}│{} {:>12} {}│{} {:>12} {}│{} {:>12} {}│",
        c_gray, c_cyan, lang.name(),
        c_gray, c_reset, format_number(files_processed),
        c_gray, c_reset, format_number(lines),
        c_gray, c_reset, format_number(code),
        c_gray, c_reset, format_number(comments),
        c_gray, c_reset, format_number(blanks),
        c_gray, c_reset, format_number(total_kw),
        c_gray
    );
    let _ = writeln!(out, "{}{}{}", c_gray, m_border_bot, c_reset);

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

    let kw_border_top = "┌────────┬────────────────────┬──────────────┬───────────┐";
    let kw_border_sep = "├────────┼────────────────────┼──────────────┼───────────┤";
    let kw_border_bot = "└────────┴────────────────────┴──────────────┴───────────┘";

    let _ = writeln!(out, "{}{}{}", c_gray, kw_border_top, c_reset);
    let _ = writeln!(
        out,
        "{}│{} {:>6} {}│{} {:<18} {}│{} {:>12} {}│{} {:>9} {}│",
        c_gray, c_white, "Rank",
        c_gray, c_white, "Keyword",
        c_gray, c_white, "Count",
        c_gray, c_white, "Percent",
        c_gray
    );
    let _ = writeln!(out, "{}{}{}", c_gray, kw_border_sep, c_reset);

    for (i, st) in stats.iter().enumerate() {
        let _ = writeln!(
            out,
            "{}│{} {:>6} {}│{} {:<18} {}│{} {:>12} {}│{} {:8.2}% {}│",
            c_gray, c_reset, i + 1,
            c_gray, c_cyan, kw_names[st.keyword_id as usize],
            c_gray, c_reset, format_number(st.count),
            c_gray, c_reset, st.percentage,
            c_gray
        );
    }

    let _ = writeln!(out, "{}{}{}", c_gray, kw_border_sep, c_reset);
    let _ = writeln!(
        out,
        "{}│{} {:<6} {}│{} {:<18} {}│{} {:>12} {}│{} {:>9} {}│",
        c_gray, c_green, "Total",
        c_gray, c_reset, "",
        c_gray, c_white, format_number(total_kw),
        c_gray, c_white, "100.00%",
        c_gray
    );
    let _ = writeln!(out, "{}{}{}", c_gray, kw_border_bot, c_reset);
    let _ = writeln!(
        out,
        "{}Processed {} files with {} worker threads (CPU cores: {}) in {:.3}s{}",
        c_gray,
        format_number(files_found),
        num_workers,
        cpu_count,
        processing_time,
        c_reset
    );

    let _ = write(stdout(), out.as_bytes());
}

fn write_summary_markdown<W: std::io::Write>(
    out: &mut W,
    totals: &WorkerResult,
    _processing_time: f64,
    _num_workers: usize,
    _cpu_count: usize,
) {
    let _ = writeln!(
        out,
        "| Language | Files | Lines | Code | Comments | Blanks | Keywords |"
    );
    let _ = writeln!(
        out,
        "|:---|---:|---:|---:|---:|---:|---:|"
    );

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

    for &(lang, files, lines, code, comments, blanks, kws) in &rows {
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} | {} | {} |",
            lang.name(),
            files,
            lines,
            code,
            comments,
            blanks,
            kws
        );
    }

    let _ = writeln!(
        out,
        "| Total | {} | {} | {} | {} | {} | {} |",
        total_files,
        total_lines,
        total_code,
        total_comments,
        total_blanks,
        total_keywords
    );
}

fn write_detailed_markdown<W: std::io::Write>(
    out: &mut W,
    totals: &WorkerResult,
    lang: LangId,
    _processing_time: f64,
    _num_workers: usize,
    _cpu_count: usize,
) {
    let idx = lang as usize;
    let files_processed = totals.files_processed[idx];
    let lines = totals.lang_stats.lines(lang);
    let code = totals.lang_stats.code_lines(lang);
    let comments = totals.lang_stats.comment_lines(lang);
    let blanks = totals.lang_stats.blank_lines(lang);
    let total_kw = totals.lang_stats.total_keywords(lang);
    let kw_names = LangStats::keyword_names(lang);
    let kw_counts = totals.lang_stats.keyword_counts(lang);

    let _ = writeln!(
        out,
        "### {}\n",
        lang.name()
    );
    let _ = writeln!(
        out,
        "| Language | Files | Lines | Code | Comments | Blanks | Keywords |"
    );
    let _ = writeln!(
        out,
        "|:---|---:|---:|---:|---:|---:|---:|"
    );
    let _ = writeln!(
        out,
        "| {} | {} | {} | {} | {} | {} | {} |\n",
        lang.name(),
        files_processed,
        lines,
        code,
        comments,
        blanks,
        total_kw
    );

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

    let _ = writeln!(out, "| Rank | Keyword | Count | Percent |");
    let _ = writeln!(out, "|---:|:---|---:|---:|");
    for (i, st) in stats.iter().enumerate() {
        let _ = writeln!(
            out,
            "| {} | {} | {} | {:.2}% |",
            i + 1,
            kw_names[st.keyword_id as usize],
            st.count,
            st.percentage
        );
    }
    let _ = writeln!(out, "| Total | | {} | 100.00% |", total_kw);
}

fn write_summary_csv<W: std::io::Write>(
    out: &mut W,
    totals: &WorkerResult,
) {
    let _ = writeln!(
        out,
        "language,files,lines,code,comments,blanks,keywords"
    );

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

    for &(lang, files, lines, code, comments, blanks, kws) in &rows {
        let _ = writeln!(
            out,
            "{},{},{},{},{},{},{}",
            lang.name(),
            files,
            lines,
            code,
            comments,
            blanks,
            kws
        );
    }

    let _ = writeln!(
        out,
        "Total,{},{},{},{},{},{}",
        total_files,
        total_lines,
        total_code,
        total_comments,
        total_blanks,
        total_keywords
    );
}

fn write_detailed_csv<W: std::io::Write>(
    out: &mut W,
    totals: &WorkerResult,
    lang: LangId,
) {
    let idx = lang as usize;
    let files_processed = totals.files_processed[idx];
    let lines = totals.lang_stats.lines(lang);
    let code = totals.lang_stats.code_lines(lang);
    let comments = totals.lang_stats.comment_lines(lang);
    let blanks = totals.lang_stats.blank_lines(lang);
    let total_kw = totals.lang_stats.total_keywords(lang);
    let kw_names = LangStats::keyword_names(lang);
    let kw_counts = totals.lang_stats.keyword_counts(lang);

    let _ = writeln!(
        out,
        "language,files,lines,code,comments,blanks,keywords"
    );
    let _ = writeln!(
        out,
        "{},{},{},{},{},{},{}",
        lang.name(),
        files_processed,
        lines,
        code,
        comments,
        blanks,
        total_kw
    );
    let _ = writeln!(out);

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

    let _ = writeln!(out, "rank,keyword,count,percent");
    for (i, st) in stats.iter().enumerate() {
        let _ = writeln!(
            out,
            "{},{},{},{:.2}%",
            i + 1,
            kw_names[st.keyword_id as usize],
            st.count,
            st.percentage
        );
    }
}

fn write_summary_json<W: std::io::Write>(
    out: &mut W,
    totals: &WorkerResult,
    processing_time: f64,
    num_workers: usize,
    cpu_count: usize,
) {
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

    let _ = writeln!(out, "{{");
    let _ = writeln!(out, "  \"summary\": {{");
    let _ = writeln!(out, "    \"languages\": [");
    for (i, (lang, files, lines, code, comments, blanks, kws)) in rows.iter().enumerate() {
        let comma = if i + 1 < rows.len() { "," } else { "" };
        let _ = writeln!(out, "      {{");
        let _ = writeln!(out, "        \"language\": \"{}\",", lang.name());
        let _ = writeln!(out, "        \"files\": {files},");
        let _ = writeln!(out, "        \"lines\": {lines},");
        let _ = writeln!(out, "        \"code\": {code},");
        let _ = writeln!(out, "        \"comments\": {comments},");
        let _ = writeln!(out, "        \"blanks\": {blanks},");
        let _ = writeln!(out, "        \"keywords\": {kws}");
        let _ = writeln!(out, "      }}{comma}");
    }
    let _ = writeln!(out, "    ],");
    let _ = writeln!(out, "    \"total\": {{");
    let _ = writeln!(out, "      \"files\": {total_files},");
    let _ = writeln!(out, "      \"lines\": {total_lines},");
    let _ = writeln!(out, "      \"code\": {total_code},");
    let _ = writeln!(out, "      \"comments\": {total_comments},");
    let _ = writeln!(out, "      \"blanks\": {total_blanks},");
    let _ = writeln!(out, "      \"keywords\": {total_keywords}");
    let _ = writeln!(out, "    }}");
    let _ = writeln!(out, "  }},");
    let _ = writeln!(out, "  \"meta\": {{");
    let _ = writeln!(out, "    \"total_files_found\": {},", totals.total_files_found());
    let _ = writeln!(out, "    \"workers\": {num_workers},");
    let _ = writeln!(out, "    \"cpu_cores\": {cpu_count},");
    let _ = writeln!(out, "    \"processing_time_seconds\": {processing_time}");
    let _ = writeln!(out, "  }}");
    let _ = writeln!(out, "}}");
}

fn write_detailed_json<W: std::io::Write>(
    out: &mut W,
    totals: &WorkerResult,
    selected_langs: &[LangId],
    processing_time: f64,
    num_workers: usize,
    cpu_count: usize,
) {
    let _ = writeln!(out, "{{");
    let _ = writeln!(out, "  \"languages\": [");
    for (l_idx, &lang) in selected_langs.iter().enumerate() {
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

        let comma_l = if l_idx + 1 < selected_langs.len() { "," } else { "" };
        let _ = writeln!(out, "    {{");
        let _ = writeln!(out, "      \"language\": \"{}\",", lang.name());
        let _ = writeln!(out, "      \"files_found\": {files_found},");
        let _ = writeln!(out, "      \"files_processed\": {files_processed},");
        let _ = writeln!(out, "      \"lines\": {lines},");
        let _ = writeln!(out, "      \"code\": {code},");
        let _ = writeln!(out, "      \"comments\": {comments},");
        let _ = writeln!(out, "      \"blanks\": {blanks},");
        let _ = writeln!(out, "      \"keywords\": {total_kw},");
        let _ = writeln!(out, "      \"keyword_frequencies\": [");
        for (k_idx, st) in stats.iter().enumerate() {
            let comma_k = if k_idx + 1 < stats.len() { "," } else { "" };
            let _ = writeln!(out, "        {{");
            let _ = writeln!(out, "          \"rank\": {},", k_idx + 1);
            let _ = writeln!(out, "          \"keyword\": \"{}\",", kw_names[st.keyword_id as usize]);
            let _ = writeln!(out, "          \"count\": {},", st.count);
            let _ = writeln!(out, "          \"percent\": {:.2}", st.percentage);
            let _ = writeln!(out, "        }}{comma_k}");
        }
        let _ = writeln!(out, "      ]");
        let _ = writeln!(out, "    }}{comma_l}");
    }
    let _ = writeln!(out, "  ],");
    let _ = writeln!(out, "  \"meta\": {{");
    let _ = writeln!(out, "    \"total_files_found\": {},", totals.total_files_found());
    let _ = writeln!(out, "    \"workers\": {num_workers},");
    let _ = writeln!(out, "    \"cpu_cores\": {cpu_count},");
    let _ = writeln!(out, "    \"processing_time_seconds\": {processing_time}");
    let _ = writeln!(out, "  }}");
    let _ = writeln!(out, "}}");
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

    for &(lang, files, lines, code, comments, blanks, kws) in &rows {
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

    let has_lang_rows = !rows.is_empty();
    if has_lang_rows {
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
    } else {
        out.clear();
    }

    let mut doc_rows: Vec<(DocFormatId, i64, i64, i64, i64, i64)> = Vec::with_capacity(DOC_FORMAT_COUNT);
    let mut doc_total_files = 0i64;
    let mut doc_total_lines = 0i64;
    let mut doc_total_content = 0i64;
    let mut doc_total_comments = 0i64;
    let mut doc_total_blanks = 0i64;

    for doc in ALL_DOC_FORMATS {
        let idx = doc as usize;
        let files = totals.doc_files_processed[idx];
        if files > 0 {
            let st = &totals.doc_stats.stats[idx];
            doc_total_files += files;
            doc_total_lines += st.lines;
            doc_total_content += st.content_lines;
            doc_total_comments += st.comment_lines;
            doc_total_blanks += st.blank_lines;
            doc_rows.push((doc, files, st.lines, st.content_lines, st.comment_lines, st.blank_lines));
        }
    }

    if !doc_rows.is_empty() {
        doc_rows.sort_unstable_by(|a, b| b.2.cmp(&a.2).then_with(|| a.0.name().cmp(b.0.name())));

        let d_border_top = "┌────────────────────┬──────────┬──────────────┬──────────────┬──────────────┬──────────────┐";
        let d_border_sep = "├────────────────────┼──────────┼──────────────┼──────────────┼──────────────┼──────────────┤";
        let d_border_bot = "└────────────────────┴──────────┴──────────────┴──────────────┴──────────────┴──────────────┘";

        let _ = writeln!(out, "{}{}{}", c_gray, d_border_top, c_reset);
        let _ = writeln!(
            out,
            "{}│{} {:<18} {}│{} {:>8} {}│{} {:>12} {}│{} {:>12} {}│{} {:>12} {}│{} {:>12} {}│",
            c_gray, c_white, "Document / Data",
            c_gray, c_white, "Files",
            c_gray, c_white, "Lines",
            c_gray, c_white, "Content",
            c_gray, c_white, "Comments",
            c_gray, c_white, "Blanks",
            c_gray
        );
        let _ = writeln!(out, "{}{}{}", c_gray, d_border_sep, c_reset);

        for (doc, files, lines, content, comments, blanks) in doc_rows {
            let _ = writeln!(
                out,
                "{}│{} {:<18} {}│{} {:>8} {}│{} {:>12} {}│{} {:>12} {}│{} {:>12} {}│{} {:>12} {}│",
                c_gray, c_cyan, doc.name(),
                c_gray, c_reset, format_number(files),
                c_gray, c_reset, format_number(lines),
                c_gray, c_reset, format_number(content),
                c_gray, c_reset, format_number(comments),
                c_gray, c_reset, format_number(blanks),
                c_gray
            );
        }

        let _ = writeln!(out, "{}{}{}", c_gray, d_border_sep, c_reset);
        let _ = writeln!(
            out,
            "{}│{} {:<18} {}│{} {:>8} {}│{} {:>12} {}│{} {:>12} {}│{} {:>12} {}│{} {:>12} {}│",
            c_gray, c_green, "Total",
            c_gray, c_white, format_number(doc_total_files),
            c_gray, c_white, format_number(doc_total_lines),
            c_gray, c_white, format_number(doc_total_content),
            c_gray, c_white, format_number(doc_total_comments),
            c_gray, c_white, format_number(doc_total_blanks),
            c_gray
        );
        let _ = writeln!(out, "{}{}{}", c_gray, d_border_bot, c_reset);
    }

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

fn print_version() {
    let color = should_color();
    let (c_bold, c_cyan, c_gray, c_reset) = if color {
        ("\x1b[1m", "\x1b[36m", "\x1b[90m", "\x1b[0m")
    } else {
        ("", "", "", "")
    };
    println!(
        "{}{}lfa{} {}v{}{}",
        c_bold,
        c_cyan,
        c_reset,
        c_gray,
        env!("CARGO_PKG_VERSION"),
        c_reset
    );
}

fn render_usage<W: std::io::Write>(writer: &mut W, prog: &str) {
    let color = should_color();
    let (c_bold, c_cyan, c_yellow, c_gray, c_reset) = if color {
        ("\x1b[1m", "\x1b[36m", "\x1b[33m", "\x1b[90m", "\x1b[0m")
    } else {
        ("", "", "", "", "")
    };

    let _ = writeln!(
        writer,
        "{}{}lfa{} {}v{}{}\nHigh-performance parallel source code keyword analyzer\n",
        c_bold,
        c_cyan,
        c_reset,
        c_gray,
        env!("CARGO_PKG_VERSION"),
        c_reset
    );
    let _ = writeln!(
        writer,
        "{c_bold}USAGE:{c_reset}\n    {prog} [OPTIONS] <PATH>\n"
    );
    let _ = writeln!(writer, "{c_bold}ARGUMENTS:{c_reset}");
    let _ = writeln!(
        writer,
        "    {c_yellow}<PATH>{c_reset}\n            Source file or root directory to recursively analyze\n"
    );
    let _ = writeln!(writer, "{c_bold}OPTIONS:{c_reset}");
    let _ = writeln!(
        writer,
        "    {c_cyan}-h{c_reset}, {c_cyan}--help{c_reset}\n            Print help information and supported languages\n"
    );
    let _ = writeln!(
        writer,
        "    {c_cyan}-V{c_reset}, {c_cyan}--version{c_reset}\n            Print version information\n"
    );
    let _ = writeln!(
        writer,
        "    {c_cyan}-f{c_reset}, {c_cyan}--format <FORMAT>{c_reset}\n            Output format: table, json, csv, markdown (or md)\n            Short flags: {c_cyan}--json{c_reset}, {c_cyan}--csv{c_reset}, {c_cyan}--markdown{c_reset}\n"
    );
    let _ = writeln!(
        writer,
        "    {c_cyan}--hidden{c_reset}\n            Count hidden files and directories (starting with '.')\n"
    );
    let _ = writeln!(writer, "{c_bold}LANGUAGE FILTERS:{c_reset}");
    let _ = writeln!(
        writer,
        "    Filter analysis and display breakdown table for specific language(s).\n"
    );

    let mut max_width = 0;
    for lang in ALL_LANGS {
        let aliases = lang.flag_aliases();
        let width = aliases.iter().map(|s| s.len()).sum::<usize>() + (aliases.len().saturating_sub(1) * 2);
        if width > max_width {
            max_width = width;
        }
    }

    for lang in ALL_LANGS {
        let aliases = lang.flag_aliases();
        let flags_str = aliases.join(", ");
        let _ = writeln!(
            writer,
            "    {c_cyan}{flags_str:<width$}{c_reset}  Show detailed {} keyword frequencies",
            lang.name(),
            width = max_width
        );
    }

    let _ = writeln!(writer, "\n{c_bold}EXAMPLES:{c_reset}");
    let _ = writeln!(
        writer,
        "    {c_gray}# Scan entire repository for all {} supported languages{c_reset}",
        LANG_COUNT
    );
    let _ = writeln!(writer, "    {prog} .\n");
    let _ = writeln!(
        writer,
        "    {c_gray}# Filter for Rust and C++ with frequency breakdown{c_reset}"
    );
    let _ = writeln!(writer, "    {prog} -rust -cpp /path/to/code");
}

fn print_usage(prog: &str) {
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    render_usage(&mut handle, prog);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
enum OutputFormat {
    #[default]
    Table,
    Json,
    Csv,
    Markdown,
}

impl OutputFormat {
    fn from_str_name(s: &str) -> Option<Self> {
        if s.eq_ignore_ascii_case("table") {
            Some(OutputFormat::Table)
        } else if s.eq_ignore_ascii_case("json") {
            Some(OutputFormat::Json)
        } else if s.eq_ignore_ascii_case("csv") {
            Some(OutputFormat::Csv)
        } else if s.eq_ignore_ascii_case("markdown") || s.eq_ignore_ascii_case("md") {
            Some(OutputFormat::Markdown)
        } else {
            None
        }
    }
}

enum CliResult<'a> {
    Run {
        target: &'a OsStr,
        mask: u128,
        selected_langs: Vec<LangId>,
        format: OutputFormat,
        hidden: bool,
    },
    Help,
    Version,
    Err(String),
}

fn try_parse_cli_args<'a>(args: &'a [std::ffi::OsString]) -> CliResult<'a> {
    if args.len() < 2 {
        return CliResult::Err("Missing target path <PATH>".to_string());
    }

    let mut target_os: Option<&'a OsStr> = None;
    let mut flag_mask = 0u128;
    let mut selected_langs: Vec<LangId> = Vec::new();
    let mut format = OutputFormat::Table;
    let mut hidden = false;

    let mut idx = 1;
    while idx < args.len() {
        let arg = &args[idx];
        let bytes = arg.as_bytes();
        if bytes == b"-h" || bytes == b"--help" {
            return CliResult::Help;
        }
        if bytes == b"-V" || bytes == b"--version" {
            return CliResult::Version;
        }

        if bytes == b"-f" || bytes == b"--format" {
            idx += 1;
            if idx >= args.len() {
                return CliResult::Err(format!("Missing argument for '{}'", arg.to_string_lossy()));
            }
            let val = args[idx].to_string_lossy();
            match OutputFormat::from_str_name(&val) {
                Some(fmt) => format = fmt,
                None => return CliResult::Err(format!("Unknown output format '{val}'")),
            }
        } else if bytes.starts_with(b"--format=") {
            let val = String::from_utf8_lossy(&bytes[b"--format=".len()..]);
            match OutputFormat::from_str_name(&val) {
                Some(fmt) => format = fmt,
                None => return CliResult::Err(format!("Unknown output format '{val}'")),
            }
        } else if bytes == b"--json" {
            format = OutputFormat::Json;
        } else if bytes == b"--csv" {
            format = OutputFormat::Csv;
        } else if bytes == b"--markdown" || bytes == b"--md" {
            format = OutputFormat::Markdown;
        } else if bytes == b"--hidden" {
            hidden = true;
        } else if bytes.starts_with(b"-") && bytes.len() > 1 {
            if let Some(lang) = LangId::from_flag(bytes) {
                flag_mask |= lang.mask();
                if !selected_langs.contains(&lang) {
                    selected_langs.push(lang);
                }
                if lang == LangId::C {
                    flag_mask |= LangId::CHeader.mask();
                    if !selected_langs.contains(&LangId::CHeader) {
                        selected_langs.push(LangId::CHeader);
                    }
                } else if lang == LangId::Cpp {
                    flag_mask |= LangId::CppHeader.mask();
                    if !selected_langs.contains(&LangId::CppHeader) {
                        selected_langs.push(LangId::CppHeader);
                    }
                }
            } else {
                return CliResult::Err(format!("Unknown flag '{}'", arg.to_string_lossy()));
            }
        } else if target_os.is_none() {
            target_os = Some(arg.as_os_str());
        } else {
            return CliResult::Err(format!("Unexpected argument '{}'", arg.to_string_lossy()));
        }

        idx += 1;
    }

    let Some(target) = target_os else {
        return CliResult::Err("Missing target path <PATH>".to_string());
    };

    let enabled_mask = if flag_mask != 0 {
        flag_mask
    } else {
        LANG_ALL_MASK
    };

    CliResult::Run {
        target,
        mask: enabled_mask,
        selected_langs,
        format,
        hidden,
    }
}

fn print_error_and_exit(prog: &str, msg: &str) -> ! {
    let color = should_color();
    let (c_bold_red, c_bold, c_cyan, c_reset) = if color {
        ("\x1b[1;31m", "\x1b[1m", "\x1b[36m", "\x1b[0m")
    } else {
        ("", "", "", "")
    };

    eprintln!("{c_bold_red}error:{c_reset} {c_bold}{msg}{c_reset}\n");
    eprintln!("For more information, try '{c_cyan}{prog} --help{c_reset}'.");
    process::exit(2);
}

fn parse_cli_args(args: &[std::ffi::OsString]) -> (&OsStr, u128, Vec<LangId>, OutputFormat, bool) {
    let prog = args
        .first()
        .map(|s| s.to_string_lossy())
        .unwrap_or_else(|| "lfa".into());

    match try_parse_cli_args(args) {
        CliResult::Run {
            target,
            mask,
            selected_langs,
            format,
            hidden,
        } => (target, mask, selected_langs, format, hidden),
        CliResult::Help => {
            print_usage(&prog);
            process::exit(0);
        }
        CliResult::Version => {
            print_version();
            process::exit(0);
        }
        CliResult::Err(e) => {
            print_error_and_exit(&prog, &e);
        }
    }
}

fn main() {
    let start = Instant::now();
    let args: Vec<_> = env::args_os().collect();
    let (target_os, enabled_mask, selected_langs, output_format, hidden) = parse_cli_args(&args);
    let target_bytes = target_os.as_bytes();

    let Ok(stx) = statx(CWD, target_bytes, AtFlags::empty(), StatxFlags::TYPE) else {
        let color = should_color();
        let (c_bold_red, c_bold, c_yellow, c_reset) = if color {
            ("\x1b[1;31m", "\x1b[1m", "\x1b[33m", "\x1b[0m")
        } else {
            ("", "", "", "")
        };
        eprintln!(
            "{c_bold_red}error:{c_reset} {c_bold}Cannot access path {c_yellow}'{}'{c_reset}",
            target_os.to_string_lossy()
        );
        eprintln!("Please check that the file or directory exists and you have permission to read it.");
        process::exit(1);
    };

    let ft = FileType::from_raw_mode(RawMode::from(stx.stx_mode));
    let cpu_count = get_cpu_count();
    let mut totals = WorkerResult::default();
    let mut num_workers = cpu_count;

    if ft == FileType::Directory {
        let sched = WorkScheduler::new(target_bytes, enabled_mask, hidden);
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
        let mut target_nul = Vec::with_capacity(target_bytes.len() + 1);
        target_nul.extend_from_slice(target_bytes);
        target_nul.push(0);
        let target_cstr = CStr::from_bytes_with_nul(&target_nul).unwrap();

        let mut lang_opt = if ft == FileType::RegularFile {
            detect_language(target_bytes, enabled_mask).or_else(|| {
                if !target_bytes.contains(&b'.') {
                    let mut probe_buf = [0u8; 128];
                    if let Ok(file_fd) = openat(CWD, target_cstr, OFlags::RDONLY | OFlags::CLOEXEC, Mode::empty())
                        && let Ok(n) = read(&file_fd, &mut probe_buf)
                    {
                        return detect_shebang(&probe_buf[..n], enabled_mask);
                    }
                }
                None
            })
        } else {
            None
        };

        if let Some(LangId::D) = lang_opt
            && target_bytes.ends_with(b".d")
        {
            let mut probe_buf = [0u8; 128];
            if let Ok(file_fd) = openat(CWD, target_cstr, OFlags::RDONLY | OFlags::CLOEXEC, Mode::empty())
                && let Ok(n) = read(&file_fd, &mut probe_buf)
                && lang::d::is_d_dep_file(&probe_buf[..n])
            {
                lang_opt = None;
            }
        }

        let Some(mut lang) = lang_opt else {
            eprintln!(
                "Error: {} is not a supported source file",
                target_os.to_string_lossy()
            );
            process::exit(1);
        };
        num_workers = 1;
        let mut fb = Vec::with_capacity(131072);
        if read_file_into(CWD, target_cstr, &mut fb) {
            if lang == LangId::Matlab
                && (enabled_mask & LANG_OBJECTIVEC_MASK) != 0
                && lang::objc::is_objc_file(&fb)
            {
                lang = LangId::ObjectiveC;
            }
            let idx = lang as usize;
            totals.files_found[idx] = 1;
            totals.files_processed[idx] = 1;
            parse_into(lang, &fb, &mut totals.lang_stats);
        } else {
            totals.files_found[lang as usize] = 1;
        }
    }

    if totals.total_files_found() == 0 {
        println!("No supported source files found");
        process::exit(1);
    }

    let processing_time = start.elapsed().as_secs_f64();
    match output_format {
        OutputFormat::Table => {
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
        OutputFormat::Markdown => {
            let stdout = std::io::stdout();
            let mut handle = stdout.lock();
            if selected_langs.is_empty() {
                write_summary_markdown(&mut handle, &totals, processing_time, num_workers, cpu_count);
            } else {
                for (i, &lang) in selected_langs.iter().enumerate() {
                    if i > 0 {
                        let _ = writeln!(handle);
                    }
                    write_detailed_markdown(&mut handle, &totals, lang, processing_time, num_workers, cpu_count);
                }
            }
        }
        OutputFormat::Csv => {
            let stdout = std::io::stdout();
            let mut handle = stdout.lock();
            if selected_langs.is_empty() {
                write_summary_csv(&mut handle, &totals);
            } else {
                for (i, &lang) in selected_langs.iter().enumerate() {
                    if i > 0 {
                        let _ = writeln!(handle);
                    }
                    write_detailed_csv(&mut handle, &totals, lang);
                }
            }
        }
        OutputFormat::Json => {
            let stdout = std::io::stdout();
            let mut handle = stdout.lock();
            if selected_langs.is_empty() {
                write_summary_json(&mut handle, &totals, processing_time, num_workers, cpu_count);
            } else {
                write_detailed_json(&mut handle, &totals, &selected_langs, processing_time, num_workers, cpu_count);
            }
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

        let sched = WorkScheduler::new(tmp_dir.as_bytes(), LANG_ALL_MASK, true);
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
        let ch_idx = LangId::CHeader as usize;
        let cpp_idx = LangId::Cpp as usize;
        assert_eq!(totals.files_found[c_idx], 2);
        assert_eq!(totals.files_processed[c_idx], 2);
        assert_eq!(totals.files_found[ch_idx], 1);
        assert_eq!(totals.files_processed[ch_idx], 1);
        assert_eq!(totals.lang_stats.c.lines_processed, 2);
        assert_eq!(totals.lang_stats.c_header.lines_processed, 1);
        assert_eq!(totals.lang_stats.c.total_keywords, 11);
        assert_eq!(totals.lang_stats.c_header.total_keywords, 4);
        assert_eq!(totals.lang_stats.c.keyword_counts[find_c_kw_id("int")], 1);
        assert_eq!(totals.lang_stats.c_header.keyword_counts[find_c_kw_id("int")], 1);
        assert_eq!(totals.lang_stats.c.keyword_counts[find_c_kw_id("void")], 2);
        assert_eq!(
            totals.lang_stats.c.keyword_counts[find_c_kw_id("return")],
            2
        );
        assert_eq!(totals.lang_stats.c.keyword_counts[find_c_kw_id("true")], 1);
        assert_eq!(totals.lang_stats.c.keyword_counts[find_c_kw_id("false")], 1);
        assert_eq!(
            totals.lang_stats.c_header.keyword_counts[find_c_kw_id("constexpr")],
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

        let c_only_sched = WorkScheduler::new(tmp_dir.as_bytes(), LangId::C.mask(), true);
        let c_only_res = unified_worker_proc(&c_only_sched);
        assert_eq!(c_only_res.files_found[c_idx], 2);
        assert_eq!(c_only_res.files_found[cpp_idx], 0);

        let ch_only_sched = WorkScheduler::new(tmp_dir.as_bytes(), LangId::CHeader.mask(), true);
        let ch_only_res = unified_worker_proc(&ch_only_sched);
        assert_eq!(ch_only_res.files_found[ch_idx], 1);
        assert_eq!(ch_only_res.files_found[c_idx], 0);

        let cpp_only_sched = WorkScheduler::new(tmp_dir.as_bytes(), LangId::Cpp.mask(), true);
        let cpp_only_res = unified_worker_proc(&cpp_only_sched);
        assert_eq!(cpp_only_res.files_found[c_idx], 0);
        assert_eq!(cpp_only_res.files_found[cpp_idx], 1);

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
    fn test_doc_format_detection_and_parsing() {
        assert_eq!(detect_doc_format(b"README"), Some(DocFormatId::PlainText));
        assert_eq!(detect_doc_format(b"LICENSE"), Some(DocFormatId::PlainText));
        assert_eq!(detect_doc_format(b"notes.txt"), Some(DocFormatId::PlainText));
        assert_eq!(detect_doc_format(b"guide.md"), Some(DocFormatId::Markdown));
        assert_eq!(detect_doc_format(b"data.csv"), Some(DocFormatId::Csv));
        assert_eq!(detect_doc_format(b"data.tsv"), Some(DocFormatId::Csv));
        assert_eq!(detect_doc_format(b"image.svg"), Some(DocFormatId::Svg));
        assert_eq!(detect_doc_format(b"config.ini"), Some(DocFormatId::Ini));
        assert_eq!(detect_doc_format(b"app.conf"), Some(DocFormatId::Ini));
        assert_eq!(detect_doc_format(b"my.service"), Some(DocFormatId::Ini));
        assert_eq!(detect_doc_format(b"my.socket"), Some(DocFormatId::Ini));
        assert_eq!(detect_doc_format(b"doc.rst"), Some(DocFormatId::RestructuredText));
        assert_eq!(detect_doc_format(b"changes.diff"), Some(DocFormatId::Patch));
        assert_eq!(detect_doc_format(b"changes.patch"), Some(DocFormatId::Patch));
        assert_eq!(detect_doc_format(b"settings.json5"), Some(DocFormatId::Json5));
        assert_eq!(detect_doc_format(b"settings.jsonc"), Some(DocFormatId::Json5));
        assert_eq!(detect_doc_format(b"unknown.xyz"), None);

        let parse_doc = |doc: DocFormatId, bytes: &[u8]| -> DocStats {
            let mut st = DocStats::default();
            parse_doc_into(doc, bytes, &mut st);
            st
        };

        let md_trailing = parse_doc(DocFormatId::Markdown, b"<!-- comment --> trailing\n");
        assert_eq!(md_trailing.lines, 1);
        assert_eq!(md_trailing.content_lines, 1);
        assert_eq!(md_trailing.comment_lines, 0);
        assert_eq!(
            md_trailing.lines,
            md_trailing.content_lines + md_trailing.comment_lines + md_trailing.blank_lines
        );

        let md_multi_trailing = parse_doc(DocFormatId::Markdown, b"<!--\ncomment --> trailing\n");
        assert_eq!(md_multi_trailing.lines, 2);
        assert_eq!(md_multi_trailing.comment_lines, 1);
        assert_eq!(md_multi_trailing.content_lines, 1);
        assert_eq!(
            md_multi_trailing.lines,
            md_multi_trailing.content_lines
                + md_multi_trailing.comment_lines
                + md_multi_trailing.blank_lines
        );

        let json5_trailing = parse_doc(DocFormatId::Json5, b"/* comment */ true\n");
        assert_eq!(json5_trailing.lines, 1);
        assert_eq!(json5_trailing.content_lines, 1);
        assert_eq!(json5_trailing.comment_lines, 0);
        assert_eq!(
            json5_trailing.lines,
            json5_trailing.content_lines
                + json5_trailing.comment_lines
                + json5_trailing.blank_lines
        );

        let json5_multi_trailing = parse_doc(DocFormatId::Json5, b"/*\ncomment */ true\n");
        assert_eq!(json5_multi_trailing.lines, 2);
        assert_eq!(json5_multi_trailing.comment_lines, 1);
        assert_eq!(json5_multi_trailing.content_lines, 1);
        assert_eq!(
            json5_multi_trailing.lines,
            json5_multi_trailing.content_lines
                + json5_multi_trailing.comment_lines
                + json5_multi_trailing.blank_lines
        );

        let rich_doc = "<!-- c -->\n<!-- c --> trailing\n<!--\nmulti\n--> tail\n// c\n/* c */\n/* c */ trailing\n/*\nmulti\n*/ tail\n# ini\n; ini2\n.. rst\n\nkey=val";
        let rich_doc_nl = format!("{rich_doc}\n");
        let rich_doc_crlf = rich_doc.replace('\n', "\r\n");
        let rich_bytes = rich_doc.as_bytes();
        for doc in ALL_DOC_FORMATS {
            let mut st = DocStats::default();
            parse_doc_into(doc, b"", &mut st);
            assert_eq!(st.lines, 0);
            assert_eq!(st.content_lines, 0);
            assert_eq!(st.comment_lines, 0);
            assert_eq!(st.blank_lines, 0);

            let mut st_content = DocStats::default();
            let sample = b"line 1

line 2
";
            parse_doc_into(doc, sample, &mut st_content);
            assert_eq!(st_content.lines, 3);
            assert_eq!(st_content.blank_lines, 1);
            assert_eq!(st_content.lines, st_content.content_lines + st_content.comment_lines + st_content.blank_lines);

            let base_st = parse_doc(doc, rich_bytes);
            assert_eq!(base_st, parse_doc(doc, rich_doc_nl.as_bytes()), "{doc:?}");
            assert_eq!(base_st, parse_doc(doc, rich_doc_crlf.as_bytes()), "{doc:?}");
            for offset in 0..=rich_bytes.len() {
                let slice = &rich_bytes[..offset];
                let trunc_st = parse_doc(doc, slice);
                let expected_lines = slice.iter().filter(|&&b| b == b'\n').count() as i64
                    + i64::from(slice.last().is_some_and(|&b| b != b'\n'));
                assert_eq!(trunc_st.lines, expected_lines, "{doc:?} at offset {offset}");
                assert_eq!(
                    trunc_st.lines,
                    trunc_st.content_lines + trunc_st.comment_lines + trunc_st.blank_lines,
                    "{doc:?} at offset {offset}"
                );
            }
        }

        let mut md_st = DocStats::default();
        parse_doc_into(DocFormatId::Markdown, b"# Header
<!-- comment -->
Content
", &mut md_st);
        assert_eq!(md_st.lines, 3);
        assert_eq!(md_st.content_lines, 2);
        assert_eq!(md_st.comment_lines, 1);

        let mut ini_st = DocStats::default();
        parse_doc_into(DocFormatId::Ini, b"# comment
; comment 2
key=val
", &mut ini_st);
        assert_eq!(ini_st.lines, 3);
        assert_eq!(ini_st.content_lines, 1);
        assert_eq!(ini_st.comment_lines, 2);
    }

    #[test]
    fn test_batch_limits_and_slash_normalization() {
        let mut batch = Batch::new();
        let mut too_long = false;
        let target_c = FileTarget::from_lang(LangId::C as u8);

        assert!(batch.append(b"/tmp/dir", b"a.c", target_c, &mut too_long));
        assert!(!too_long);
        assert_eq!(batch.get_path_cstr(0).to_bytes(), b"/tmp/dir/a.c");
        assert_eq!(batch.get_rel_name_cstr(0).to_bytes(), b"a.c");

        assert!(batch.append(b"/tmp/dir/", b"b.h", target_c, &mut too_long));
        assert!(!too_long);
        assert_eq!(batch.get_path_cstr(1).to_bytes(), b"/tmp/dir/b.h");
        assert_eq!(batch.get_rel_name_cstr(1).to_bytes(), b"b.h");

        let huge_dir = vec![b'x'; BATCH_BUF_SIZE];
        assert!(!batch.append(&huge_dir, b"c.c", target_c, &mut too_long));
        assert!(too_long);
        assert_eq!(batch.count, 2);

        batch.reset();
        for i in 0..BATCH_SIZE {
            let name = format!("f{}.c", i);
            assert!(batch.append(b"/d", name.as_bytes(), target_c, &mut too_long));
            assert!(!too_long);
        }
        assert_eq!(batch.count, BATCH_SIZE);
        assert!(!batch.append(b"/d", b"overflow.c", target_c, &mut too_long));
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

        let sched = WorkScheduler::new(tmp_dir.as_bytes(), LANG_ALL_MASK, true);
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

        let disabled_sched = WorkScheduler::new(tmp_dir.as_bytes(), 0, true);
        let disabled_res = unified_worker_proc(&disabled_sched);
        assert_eq!(disabled_res.total_files_found(), 0);

        let _ = fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn test_cli_flag_parsing() {
        let args = vec![OsString::from("lfa"), OsString::from("/tmp")];
        let (target, mask, detailed, format, hidden) = parse_cli_args(&args);
        assert_eq!(target.as_bytes(), b"/tmp");
        assert_eq!(mask, LANG_ALL_MASK);
        assert_eq!(detailed, Vec::<LangId>::new());
        assert_eq!(format, OutputFormat::Table);
        assert!(!hidden);

        let args_hidden = vec![
            OsString::from("lfa"),
            OsString::from("--hidden"),
            OsString::from("/tmp"),
        ];
        let (target, mask, detailed, format, hidden) = parse_cli_args(&args_hidden);
        assert_eq!(target.as_bytes(), b"/tmp");
        assert_eq!(mask, LANG_ALL_MASK);
        assert_eq!(detailed, Vec::<LangId>::new());
        assert_eq!(format, OutputFormat::Table);
        assert!(hidden);

        let args_c = vec![
            OsString::from("lfa"),
            OsString::from("/tmp"),
            OsString::from("-c"),
        ];
        let (target, mask, detailed, format, hidden) = parse_cli_args(&args_c);
        assert_eq!(target.as_bytes(), b"/tmp");
        assert_eq!(mask, LangId::C.mask() | LangId::CHeader.mask());
        assert_eq!(detailed, vec![LangId::C, LangId::CHeader]);
        assert_eq!(format, OutputFormat::Table);
        assert!(!hidden);

        let args_c_first = vec![
            OsString::from("lfa"),
            OsString::from("-c"),
            OsString::from("/tmp"),
        ];
        let (target, mask, detailed, format, hidden) = parse_cli_args(&args_c_first);
        assert_eq!(target.as_bytes(), b"/tmp");
        assert_eq!(mask, LangId::C.mask() | LangId::CHeader.mask());
        assert_eq!(detailed, vec![LangId::C, LangId::CHeader]);
        assert_eq!(format, OutputFormat::Table);
        assert!(!hidden);

        let args_cpp = vec![
            OsString::from("lfa"),
            OsString::from("-cpp"),
            OsString::from("/tmp"),
        ];
        let (target, mask, detailed, format, hidden) = parse_cli_args(&args_cpp);
        assert_eq!(target.as_bytes(), b"/tmp");
        assert_eq!(mask, LangId::Cpp.mask() | LangId::CppHeader.mask());
        assert_eq!(detailed, vec![LangId::Cpp, LangId::CppHeader]);
        assert_eq!(format, OutputFormat::Table);
        assert!(!hidden);

        for flag in ["-rs", "--rs", "-rust", "--rust"] {
            let args_rs = vec![
                OsString::from("lfa"),
                OsString::from(flag),
                OsString::from("/tmp"),
            ];
            let (target, mask, detailed, format, hidden) = parse_cli_args(&args_rs);
            assert_eq!(target.as_bytes(), b"/tmp");
            assert_eq!(mask, LangId::Rust.mask());
            assert_eq!(detailed, vec![LangId::Rust]);
            assert_eq!(format, OutputFormat::Table);
            assert!(!hidden);
        }

        for flag in ["-py", "--py", "-python", "--python"] {
            let args_py = vec![
                OsString::from("lfa"),
                OsString::from(flag),
                OsString::from("/tmp"),
            ];
            let (target, mask, detailed, format, hidden) = parse_cli_args(&args_py);
            assert_eq!(target.as_bytes(), b"/tmp");
            assert_eq!(mask, LangId::Python.mask());
            assert_eq!(detailed, vec![LangId::Python]);
            assert_eq!(format, OutputFormat::Table);
            assert!(!hidden);
        }

        for flag in ["-pl", "--pl", "-perl", "--perl"] {
            let args_pl = vec![
                OsString::from("lfa"),
                OsString::from(flag),
                OsString::from("/tmp"),
            ];
            let (target, mask, detailed, format, hidden) = parse_cli_args(&args_pl);
            assert_eq!(target.as_bytes(), b"/tmp");
            assert_eq!(mask, LangId::Perl.mask());
            assert_eq!(detailed, vec![LangId::Perl]);
            assert_eq!(format, OutputFormat::Table);
            assert!(!hidden);
        }

        for flag in ["-go", "--go", "-golang", "--golang"] {
            let args_go = vec![
                OsString::from("lfa"),
                OsString::from(flag),
                OsString::from("/tmp"),
            ];
            let (target, mask, detailed, format, hidden) = parse_cli_args(&args_go);
            assert_eq!(target.as_bytes(), b"/tmp");
            assert_eq!(mask, LangId::Go.mask());
            assert_eq!(detailed, vec![LangId::Go]);
            assert_eq!(format, OutputFormat::Table);
            assert!(!hidden);
        }

        for flag in ["-zig", "--zig"] {
            let args_zig = vec![
                OsString::from("lfa"),
                OsString::from(flag),
                OsString::from("/tmp"),
            ];
            let (target, mask, detailed, format, hidden) = parse_cli_args(&args_zig);
            assert_eq!(target.as_bytes(), b"/tmp");
            assert_eq!(mask, LangId::Zig.mask());
            assert_eq!(detailed, vec![LangId::Zig]);
            assert_eq!(format, OutputFormat::Table);
            assert!(!hidden);
        }

        for flag in ["-odin", "--odin"] {
            let args_odin = vec![
                OsString::from("lfa"),
                OsString::from(flag),
                OsString::from("/tmp"),
            ];
            let (target, mask, detailed, format, hidden) = parse_cli_args(&args_odin);
            assert_eq!(target.as_bytes(), b"/tmp");
            assert_eq!(mask, LangId::Odin.mask());
            assert_eq!(detailed, vec![LangId::Odin]);
            assert_eq!(format, OutputFormat::Table);
            assert!(!hidden);
        }

        for flag in ["-js", "--js", "-javascript", "--javascript"] {
            let args_js = vec![
                OsString::from("lfa"),
                OsString::from(flag),
                OsString::from("/tmp"),
            ];
            let (target, mask, detailed, format, hidden) = parse_cli_args(&args_js);
            assert_eq!(target.as_bytes(), b"/tmp");
            assert_eq!(mask, LangId::JavaScript.mask());
            assert_eq!(detailed, vec![LangId::JavaScript]);
            assert_eq!(format, OutputFormat::Table);
            assert!(!hidden);
        }

        for flag in ["-ts", "--ts", "-typescript", "--typescript"] {
            let args_ts = vec![
                OsString::from("lfa"),
                OsString::from(flag),
                OsString::from("/tmp"),
            ];
            let (target, mask, detailed, format, hidden) = parse_cli_args(&args_ts);
            assert_eq!(target.as_bytes(), b"/tmp");
            assert_eq!(mask, LangId::TypeScript.mask());
            assert_eq!(detailed, vec![LangId::TypeScript]);
            assert_eq!(format, OutputFormat::Table);
            assert!(!hidden);
        }

        for flag in ["-lua", "--lua", "-LUA", "--LUA"] {
            let args_lua = vec![
                OsString::from("lfa"),
                OsString::from(flag),
                OsString::from("/tmp"),
            ];
            let (target, mask, detailed, format, hidden) = parse_cli_args(&args_lua);
            assert_eq!(target.as_bytes(), b"/tmp");
            assert_eq!(mask, LangId::Lua.mask());
            assert_eq!(detailed, vec![LangId::Lua]);
            assert_eq!(format, OutputFormat::Table);
            assert!(!hidden);
        }

        let args_multi = vec![
            OsString::from("lfa"),
            OsString::from("-zig"),
            OsString::from("/tmp"),
            OsString::from("-c"),
            OsString::from("-zig"),
            OsString::from("-odin"),
        ];
        let (target, mask, detailed, format, hidden) = parse_cli_args(&args_multi);
        assert_eq!(target.as_bytes(), b"/tmp");
        assert_eq!(
            mask,
            LangId::Zig.mask() | LangId::C.mask() | LangId::CHeader.mask() | LangId::Odin.mask()
        );
        assert_eq!(detailed, vec![LangId::Zig, LangId::C, LangId::CHeader, LangId::Odin]);
        assert_eq!(format, OutputFormat::Table);
        assert!(!hidden);

        for lang in ALL_LANGS {
            for &alias in lang.flag_aliases() {
                if alias == "-f" {
                    continue;
                }
                let args_lang = vec![
                    OsString::from("lfa"),
                    OsString::from(alias),
                    OsString::from("/tmp"),
                ];
                let res = try_parse_cli_args(&args_lang);
                match res {
                    CliResult::Run { target, mask, selected_langs, .. } => {
                        assert_eq!(target.as_bytes(), b"/tmp");
                        let expected_mask = if lang == LangId::C {
                            LangId::C.mask() | LangId::CHeader.mask()
                        } else if lang == LangId::Cpp {
                            LangId::Cpp.mask() | LangId::CppHeader.mask()
                        } else {
                            lang.mask()
                        };
                        let expected_selected = if lang == LangId::C {
                            vec![LangId::C, LangId::CHeader]
                        } else if lang == LangId::Cpp {
                            vec![LangId::Cpp, LangId::CppHeader]
                        } else {
                            vec![lang]
                        };
                        assert_eq!(mask, expected_mask);
                        assert_eq!(selected_langs, expected_selected);
                    }
                    _ => panic!("Expected CliResult::Run for lang {:?} flag {alias}", lang),
                }
            }
        }

        assert!(matches!(
            try_parse_cli_args(&[OsString::from("lfa"), OsString::from("--help")]),
            CliResult::Help
        ));
        assert!(matches!(
            try_parse_cli_args(&[OsString::from("lfa"), OsString::from("-h")]),
            CliResult::Help
        ));
        assert!(matches!(
            try_parse_cli_args(&[OsString::from("lfa"), OsString::from("--version")]),
            CliResult::Version
        ));
        assert!(matches!(
            try_parse_cli_args(&[OsString::from("lfa"), OsString::from("-V")]),
            CliResult::Version
        ));

        match try_parse_cli_args(&[OsString::from("lfa"), OsString::from("-f"), OsString::from("json"), OsString::from("/tmp")]) {
            CliResult::Run { format, .. } => assert_eq!(format, OutputFormat::Json),
            _ => panic!("Expected CliResult::Run with json format"),
        }
        match try_parse_cli_args(&[OsString::from("lfa"), OsString::from("--format=csv"), OsString::from("/tmp")]) {
            CliResult::Run { format, .. } => assert_eq!(format, OutputFormat::Csv),
            _ => panic!("Expected CliResult::Run with csv format"),
        }
        match try_parse_cli_args(&[OsString::from("lfa"), OsString::from("--markdown"), OsString::from("/tmp")]) {
            CliResult::Run { format, .. } => assert_eq!(format, OutputFormat::Markdown),
            _ => panic!("Expected CliResult::Run with markdown format"),
        }
        match try_parse_cli_args(&[OsString::from("lfa"), OsString::from("--json"), OsString::from("/tmp")]) {
            CliResult::Run { format, .. } => assert_eq!(format, OutputFormat::Json),
            _ => panic!("Expected CliResult::Run with json format"),
        }
        match try_parse_cli_args(&[OsString::from("lfa"), OsString::from("-f")]) {
            CliResult::Err(e) => assert!(e.contains("Missing argument for '-f'")),
            _ => panic!("Expected CliResult::Err"),
        }
        match try_parse_cli_args(&[OsString::from("lfa"), OsString::from("--format"), OsString::from("yaml"), OsString::from("/tmp")]) {
            CliResult::Err(e) => assert!(e.contains("Unknown output format 'yaml'")),
            _ => panic!("Expected CliResult::Err"),
        }

        match try_parse_cli_args(&[OsString::from("lfa"), OsString::from("--unknown-flag"), OsString::from("/tmp")]) {
            CliResult::Err(e) => {
                assert!(e.contains("Unknown flag"));
                assert!(e.contains("--unknown-flag"));
            }
            _ => panic!("Expected CliResult::Err"),
        }

        match try_parse_cli_args(&[OsString::from("lfa")]) {
            CliResult::Err(e) => {
                assert!(e.contains("Missing target path"));
            }
            _ => panic!("Expected CliResult::Err"),
        }

        match try_parse_cli_args(&[OsString::from("lfa"), OsString::from("path1"), OsString::from("path2")]) {
            CliResult::Err(e) => {
                assert!(e.contains("Unexpected argument"));
                assert!(e.contains("path2"));
            }
            _ => panic!("Expected CliResult::Err"),
        }

        let mut help_buf = Vec::new();
        render_usage(&mut help_buf, "lfa");
        let help_output = String::from_utf8(help_buf).expect("valid utf8 help");
        assert!(help_output.contains("High-performance parallel source code keyword analyzer"));
        assert!(help_output.contains(&format!("all {} supported languages", LANG_COUNT)));
        assert!(help_output.contains("--format <FORMAT>"));
        assert!(help_output.contains("--json"));
        let desc_positions: Vec<usize> = help_output
            .lines()
            .filter_map(|line| line.find("Show detailed "))
            .collect();
        assert_eq!(desc_positions.len(), LANG_COUNT);
        let first_pos = desc_positions[0];
        for pos in desc_positions {
            assert_eq!(pos, first_pos);
        }

        let mut dummy_totals = WorkerResult::default();
        dummy_totals.files_processed[LangId::Rust as usize] = 2;
        dummy_totals.lang_stats.rust.lines_processed = 100;
        dummy_totals.lang_stats.rust.code_lines = 80;
        dummy_totals.lang_stats.rust.comment_lines = 10;
        dummy_totals.lang_stats.rust.blank_lines = 10;
        dummy_totals.lang_stats.rust.total_keywords = 1;
        dummy_totals.lang_stats.rust.keyword_counts[0] = 1;

        let mut csv_buf = Vec::new();
        write_summary_csv(&mut csv_buf, &dummy_totals);
        let csv_str = String::from_utf8(csv_buf).unwrap();
        assert!(csv_str.contains("Rust,2,100,80,10,10,1"));
        assert!(csv_str.contains("Total,2,100,80,10,10,1"));

        let mut json_buf = Vec::new();
        write_summary_json(&mut json_buf, &dummy_totals, 0.05, 4, 4);
        let json_str = String::from_utf8(json_buf).unwrap();
        assert!(json_str.contains("\"language\": \"Rust\""));
        assert!(json_str.contains("\"lines\": 100"));
        assert!(json_str.contains("\"code\": 80"));

        let mut md_buf = Vec::new();
        write_summary_markdown(&mut md_buf, &dummy_totals, 0.05, 4, 4);
        let md_str = String::from_utf8(md_buf).unwrap();
        assert!(md_str.contains("| Rust | 2 | 100 | 80 | 10 | 10 | 1 |"));

        let mut det_json_buf = Vec::new();
        write_detailed_json(&mut det_json_buf, &dummy_totals, &[LangId::Rust], 0.05, 4, 4);
        let det_json_str = String::from_utf8(det_json_buf).unwrap();
        assert!(det_json_str.contains("\"keyword_frequencies\": ["));
        assert!(det_json_str.contains("\"rank\": 1"));

        let mut det_csv_buf = Vec::new();
        write_detailed_csv(&mut det_csv_buf, &dummy_totals, LangId::Rust);
        let det_csv_str = String::from_utf8(det_csv_buf).unwrap();
        assert!(det_csv_str.contains("rank,keyword,count,percent"));

        let mut det_md_buf = Vec::new();
        write_detailed_markdown(&mut det_md_buf, &dummy_totals, LangId::Rust, 0.05, 4, 4);
        let det_md_str = String::from_utf8(det_md_buf).unwrap();
        assert!(det_md_str.contains("### Rust"));
        assert!(det_md_str.contains("| Rank | Keyword | Count | Percent |"));
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

    #[test]
    fn test_pipeline_single_file_and_directory_consistency() {
        let unique_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let tmp_dir = format!("/tmp/lfa_pipe_test_{unique_id}");
        fs::create_dir_all(&tmp_dir).unwrap();

        let rust_code = b"fn main() {\n    let mut x = 1;\n    // comment line\n    x += 1;\n}\n";
        let c_code = b"int main() {\n    int a = 0;\n    /* comment */\n    return a;\n}\n";

        let rust_file = format!("{tmp_dir}/main.rs");
        let c_file = format!("{tmp_dir}/main.c");
        fs::write(&rust_file, rust_code).unwrap();
        fs::write(&c_file, c_code).unwrap();

        let mut stats_dir_all = WorkerResult::default();
        let sched_all = WorkScheduler::new(tmp_dir.as_bytes(), LANG_ALL_MASK, true);
        let res_dir_all = unified_worker_proc(&sched_all);
        stats_dir_all.add(&res_dir_all);

        assert_eq!(stats_dir_all.files_processed[LangId::Rust as usize], 1);
        assert_eq!(stats_dir_all.files_processed[LangId::C as usize], 1);
        assert_eq!(stats_dir_all.lang_stats.lines(LangId::Rust), 5);
        assert_eq!(stats_dir_all.lang_stats.lines(LangId::C), 5);

        let mut stats_dir_rust = WorkerResult::default();
        let sched_rust = WorkScheduler::new(tmp_dir.as_bytes(), LangId::Rust.mask(), true);
        let res_dir_rust = unified_worker_proc(&sched_rust);
        stats_dir_rust.add(&res_dir_rust);

        assert_eq!(stats_dir_rust.files_processed[LangId::Rust as usize], 1);
        assert_eq!(stats_dir_rust.files_processed[LangId::C as usize], 0);

        let mut single_stats = LangStats::default();
        parse_into(LangId::Rust, rust_code, &mut single_stats);
        assert_eq!(stats_dir_all.lang_stats.lines(LangId::Rust), single_stats.lines(LangId::Rust));
        assert_eq!(stats_dir_all.lang_stats.code_lines(LangId::Rust), single_stats.code_lines(LangId::Rust));
        assert_eq!(stats_dir_all.lang_stats.comment_lines(LangId::Rust), single_stats.comment_lines(LangId::Rust));
        assert_eq!(stats_dir_all.lang_stats.blank_lines(LangId::Rust), single_stats.blank_lines(LangId::Rust));
        assert_eq!(stats_dir_all.lang_stats.total_keywords(LangId::Rust), single_stats.total_keywords(LangId::Rust));

        let _ = fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn test_hidden_files_and_directories_filter() {
        let unique_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let tmp_dir = format!("/tmp/lfa_hidden_test_{unique_id}");
        let hidden_dir = format!("{tmp_dir}/.hidden_dir");
        fs::create_dir_all(&hidden_dir).unwrap();

        let visible_file = format!("{tmp_dir}/visible.c");
        let hidden_file = format!("{tmp_dir}/.hidden.c");
        let in_hidden_dir_file = format!("{hidden_dir}/inside.c");

        let c_code = b"int main() { return 0; }\n";
        fs::write(&visible_file, c_code).unwrap();
        fs::write(&hidden_file, c_code).unwrap();
        fs::write(&in_hidden_dir_file, c_code).unwrap();

        let sched_default = WorkScheduler::new(tmp_dir.as_bytes(), LANG_ALL_MASK, false);
        let res_default = unified_worker_proc(&sched_default);
        assert_eq!(res_default.files_processed[LangId::C as usize], 1);
        assert_eq!(res_default.lang_stats.lines(LangId::C), 1);

        let sched_hidden = WorkScheduler::new(tmp_dir.as_bytes(), LANG_ALL_MASK, true);
        let res_hidden = unified_worker_proc(&sched_hidden);
        assert_eq!(res_hidden.files_processed[LangId::C as usize], 3);
        assert_eq!(res_hidden.lang_stats.lines(LangId::C), 3);

        let _ = fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn test_shebang_detection_in_scheduler() {
        let unique_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let tmp_dir = format!("/tmp/lfa_shebang_test_{unique_id}");
        fs::create_dir_all(&tmp_dir).unwrap();

        let bash_script = b"#!/usr/bin/env bash\necho 'hello world'\nexit 0\n";
        let python_script = b"#!/usr/bin/python3\nimport sys\nsys.stdout.write('hi')\n";
        let no_shebang = b"random plain text content without extension or shebang\n";

        let bash_path = format!("{tmp_dir}/my-tool");
        let python_path = format!("{tmp_dir}/runner");
        let plain_path = format!("{tmp_dir}/LICENSE");

        fs::write(&bash_path, bash_script).unwrap();
        fs::write(&python_path, python_script).unwrap();
        fs::write(&plain_path, no_shebang).unwrap();

        let sched = WorkScheduler::new(tmp_dir.as_bytes(), LANG_ALL_MASK, false);
        let res = unified_worker_proc(&sched);

        assert_eq!(res.files_processed[LangId::Bash as usize], 1);
        assert_eq!(res.lang_stats.lines(LangId::Bash), 3);
        assert_eq!(res.files_processed[LangId::Python as usize], 1);
        assert_eq!(res.lang_stats.lines(LangId::Python), 3);

        let _ = fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn test_dot_m_disambiguation_in_scheduler() {
        let unique_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let tmp_dir = format!("/tmp/lfa_m_disambig_{unique_id}");
        fs::create_dir_all(&tmp_dir).unwrap();

        let objc_path = format!("{tmp_dir}/cls.m");
        let matlab_path = format!("{tmp_dir}/math.m");
        fs::write(&objc_path, b"@class Foo;\n@return @YES;\n").unwrap();
        fs::write(&matlab_path, b"function y = sq(x)\ny = x * x;\nend\n").unwrap();

        let sched = WorkScheduler::new(tmp_dir.as_bytes(), LANG_ALL_MASK, false);
        let res = unified_worker_proc(&sched);

        assert_eq!(res.files_processed[LangId::ObjectiveC as usize], 1);
        assert_eq!(res.lang_stats.lines(LangId::ObjectiveC), 2);
        assert_eq!(res.lang_stats.total_keywords(LangId::ObjectiveC), 3);
        assert_eq!(res.files_processed[LangId::Matlab as usize], 1);
        assert_eq!(res.lang_stats.lines(LangId::Matlab), 3);

        let _ = fs::remove_dir_all(&tmp_dir);
    }
}
