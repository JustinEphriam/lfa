#![forbid(unsafe_code)]

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
use std::fmt::Write as _;
use std::os::unix::ffi::OsStrExt;
use std::process;
use std::sync::{Condvar, Mutex};
use std::thread;
use std::time::Instant;

const KEYWORD_COUNT: usize = 59;
const KW_SLOT_MASK: usize = 255;
const KW_MAX_LEN: usize = 14;
const KW_EMPTY: u8 = 255;

const KEYWORD_NAMES: [&str; KEYWORD_COUNT] = [
    "auto",
    "break",
    "case",
    "char",
    "const",
    "continue",
    "default",
    "do",
    "double",
    "else",
    "enum",
    "extern",
    "float",
    "for",
    "goto",
    "if",
    "int",
    "long",
    "register",
    "return",
    "short",
    "signed",
    "sizeof",
    "static",
    "struct",
    "switch",
    "typedef",
    "union",
    "unsigned",
    "void",
    "volatile",
    "while",
    "inline",
    "restrict",
    "_Bool",
    "_Complex",
    "_Imaginary",
    "_Alignas",
    "_Alignof",
    "_Atomic",
    "_Static_assert",
    "_Noreturn",
    "_Thread_local",
    "_Generic",
    "_BitInt",
    "typeof",
    "typeof_unqual",
    "_Decimal128",
    "_Decimal32",
    "_Decimal64",
    "alignas",
    "alignof",
    "bool",
    "constexpr",
    "false",
    "nullptr",
    "static_assert",
    "thread_local",
    "true",
];

const KEYWORD_LENS: [u8; KEYWORD_COUNT] = [
    4, 5, 4, 4, 5, 8, 7, 2, 6, 4, 4, 6, 5, 3, 4, 2, 3, 4, 8, 6, 5, 6, 6, 6, 6, 6, 7, 5, 8, 4, 8,
    5, 6, 8, 5, 8, 10, 8, 8, 7, 14, 9, 13, 8, 7, 6, 13, 11, 10, 10, 7, 7, 4, 9, 5, 7, 13, 12, 4,
];

const TOK_NONE: u8 = 0;
const TOK_IDENT: u8 = 1;
const TOK_SLASH: u8 = 2;
const TOK_QUOTE: u8 = 3;
const TOK_APOSTROPHE: u8 = 4;

#[inline(always)]
const fn kw_hash(first: u8, last: u8, len: usize) -> usize {
    ((first as usize)
        .wrapping_add((last as usize).wrapping_mul(9))
        .wrapping_add(len.wrapping_mul(11)))
        & KW_SLOT_MASK
}

const KW_TABLE: [u8; 256] = {
    let mut table = [KW_EMPTY; 256];
    let mut i = 0;
    while i < KEYWORD_COUNT {
        let name = KEYWORD_NAMES[i].as_bytes();
        let len = KEYWORD_LENS[i] as usize;
        assert!(name.len() == len);
        assert!(len >= 2 && len <= KW_MAX_LEN);
        let h = kw_hash(name[0], name[len - 1], len);
        assert!(table[h] == KW_EMPTY);
        table[h] = i as u8;
        i += 1;
    }
    table
};

const TOKEN_KIND: [u8; 256] = {
    let mut table = [TOK_NONE; 256];
    let mut c = b'a';
    while c <= b'z' {
        table[c as usize] = TOK_IDENT;
        c += 1;
    }
    let mut c = b'A';
    while c <= b'Z' {
        table[c as usize] = TOK_IDENT;
        c += 1;
    }
    table[b'_' as usize] = TOK_IDENT;
    table[b'/' as usize] = TOK_SLASH;
    table[b'"' as usize] = TOK_QUOTE;
    table[b'\'' as usize] = TOK_APOSTROPHE;
    table
};

const IDENT_CONT: [u8; 256] = {
    let mut table = [0u8; 256];
    let mut c = b'a';
    while c <= b'z' {
        table[c as usize] = 1;
        c += 1;
    }
    let mut c = b'A';
    while c <= b'Z' {
        table[c as usize] = 1;
        c += 1;
    }
    let mut c = b'0';
    while c <= b'9' {
        table[c as usize] = 1;
        c += 1;
    }
    table[b'_' as usize] = 1;
    table
};

#[inline(always)]
fn count_newlines_avx2(bytes: &[u8]) -> usize {
    #[cfg(target_feature = "avx2")]
    {
        use safe_arch::*;
        let target = set_splat_i8_m256i(b'\n' as i8);
        let zero = zeroed_m256i();
        let mut count = 0usize;
        let (chunks, remainder) = bytes.as_chunks::<32>();
        for block in chunks.chunks(255) {
            let mut acc = zeroed_m256i();
            for &chunk in block {
                let v = m256i::from(chunk);
                let cmp = cmp_eq_mask_i8_m256i(v, target);
                acc = sub_i8_m256i(acc, cmp);
            }
            let sad: [u64; 4] = sum_of_u8_abs_diff_m256i(acc, zero).into();
            count += (sad[0] + sad[1] + sad[2] + sad[3]) as usize;
        }
        for &b in remainder {
            if b == b'\n' {
                count += 1;
            }
        }
        count
    }
    #[cfg(not(target_feature = "avx2"))]
    {
        memchr::memchr_iter(b'\n', bytes).count()
    }
}

#[inline(always)]
fn find_byte_avx2(slice: &[u8], needle: u8) -> Option<usize> {
    #[cfg(target_feature = "avx2")]
    {
        use safe_arch::*;
        let target = set_splat_i8_m256i(needle as i8);
        let (chunks, remainder) = slice.as_chunks::<32>();
        let mut i = 0usize;
        for &chunk in chunks {
            let v = m256i::from(chunk);
            let cmp = cmp_eq_mask_i8_m256i(v, target);
            let mask = move_mask_i8_m256i(cmp) as u32;
            if mask != 0 {
                return Some(i + mask.trailing_zeros() as usize);
            }
            i += 32;
        }
        for (idx, &b) in remainder.iter().enumerate() {
            if b == needle {
                return Some(i + idx);
            }
        }
        None
    }
    #[cfg(not(target_feature = "avx2"))]
    {
        memchr::memchr(needle, slice)
    }
}

#[inline(always)]
fn find_star_slash(slice: &[u8]) -> Option<usize> {
    if slice.len() < 2 {
        return None;
    }
    #[cfg(target_feature = "avx2")]
    {
        use safe_arch::*;
        let stars = set_splat_i8_m256i(b'*' as i8);
        let slashes = set_splat_i8_m256i(b'/' as i8);
        let mut i = 0usize;
        let (chunks0, _) = slice[..slice.len() - 1].as_chunks::<32>();
        let (chunks1, _) = slice[1..].as_chunks::<32>();
        for (&c0_arr, &c1_arr) in chunks0.iter().zip(chunks1.iter()) {
            let c0 = m256i::from(c0_arr);
            let c1 = m256i::from(c1_arr);
            let eq_star = cmp_eq_mask_i8_m256i(c0, stars);
            let eq_slash = cmp_eq_mask_i8_m256i(c1, slashes);
            let mask = move_mask_i8_m256i(eq_star & eq_slash) as u32;
            if mask != 0 {
                return Some(i + mask.trailing_zeros() as usize);
            }
            i += 32;
        }
        while i + 1 < slice.len() {
            if slice[i] == b'*' && slice[i + 1] == b'/' {
                return Some(i);
            }
            i += 1;
        }
        None
    }
    #[cfg(not(target_feature = "avx2"))]
    {
        memchr::memmem::find(slice, b"*/")
    }
}

#[inline(always)]
fn skip_line_comment(content: &[u8], start_i: usize) -> usize {
    let content_len = content.len();
    let mut i = start_i;
    while i < content_len {
        let Some(pos) = find_byte_avx2(&content[i..], b'\n') else {
            return content_len;
        };
        let nl_idx = i + pos;
        let mut check = nl_idx;
        if check > start_i && content[check - 1] == b'\r' {
            check -= 1;
        }
        if check > start_i && content[check - 1] == b'\\' {
            i = nl_idx + 1;
            continue;
        }
        return nl_idx + 1;
    }
    content_len
}

#[inline(always)]
fn skip_literal(content: &[u8], start_i: usize, quote: u8) -> usize {
    let content_len = content.len();
    let mut i = start_i;
    while i < content_len {
        let Some(pos) = find_byte_avx2(&content[i..], quote) else {
            return content_len;
        };
        i += pos;
        let mut escapes = 0usize;
        let mut j = i;
        while j > start_i && content[j - 1] == b'\\' {
            escapes += 1;
            j -= 1;
        }
        if (escapes & 1) == 0 {
            return i + 1;
        }
        i += 1;
    }
    content_len
}

#[inline(always)]
fn kw_match_len_2_to_14(cand: &[u8], kw: &[u8]) -> bool {
    let n = cand.len();
    if n != kw.len() || !(2..=KW_MAX_LEN).contains(&n) {
        return false;
    }
    if n >= 8 {
        let a1 = u64::from_ne_bytes(cand[..8].try_into().unwrap());
        let b1 = u64::from_ne_bytes(kw[..8].try_into().unwrap());
        let a2 = u64::from_ne_bytes(cand[n - 8..n].try_into().unwrap());
        let b2 = u64::from_ne_bytes(kw[n - 8..n].try_into().unwrap());
        ((a1 ^ b1) | (a2 ^ b2)) == 0
    } else if n >= 4 {
        let a1 = u32::from_ne_bytes(cand[..4].try_into().unwrap());
        let b1 = u32::from_ne_bytes(kw[..4].try_into().unwrap());
        let a2 = u32::from_ne_bytes(cand[n - 4..n].try_into().unwrap());
        let b2 = u32::from_ne_bytes(kw[n - 4..n].try_into().unwrap());
        ((a1 ^ b1) | (a2 ^ b2)) == 0
    } else {
        let a1 = u16::from_ne_bytes(cand[..2].try_into().unwrap());
        let b1 = u16::from_ne_bytes(kw[..2].try_into().unwrap());
        a1 == b1 && cand[n - 1] == kw[n - 1]
    }
}

#[inline(always)]
fn skip_non_tokens_avx2(ptr: &[u8], mut i: usize) -> usize {
    #[cfg(target_feature = "avx2")]
    {
        use safe_arch::*;
        let lut_lo = m256i::from([
            2_i8, 3, 11, 3, 3, 3, 3, 11, 3, 3, 3, 1, 1, 1, 1, 13, 2, 3, 11, 3, 3, 3, 3, 11, 3, 3,
            3, 1, 1, 1, 1, 13,
        ]);
        let lut_hi = m256i::from([
            0_i8, 0, 8, 0, 1, 6, 1, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 0, 1, 6, 1, 2, 0, 0, 0, 0,
            0, 0, 0, 0,
        ]);
        let mask_0f = set_splat_i8_m256i(0x0F);
        let zero = zeroed_m256i();

        let (chunks, _) = ptr[i..].as_chunks::<32>();
        for &chunk in chunks {
            let v = m256i::from(chunk);
            let hi = shr_imm_u16_m256i::<4>(v) & mask_0f;
            let m_lo = shuffle_av_i8z_half_m256i(lut_lo, v);
            let m_hi = shuffle_av_i8z_half_m256i(lut_hi, hi);
            let hit = m_lo & m_hi;
            let is_zero = cmp_eq_mask_i8_m256i(hit, zero);
            let mask = !(move_mask_i8_m256i(is_zero) as u32);
            if mask != 0 {
                return i + mask.trailing_zeros() as usize;
            }
            i += 32;
        }
    }
    while i < ptr.len() && TOKEN_KIND[ptr[i] as usize] == TOK_NONE {
        i += 1;
    }
    i
}

#[inline(always)]
fn skip_ident_cont_avx2(ptr: &[u8], mut i: usize) -> usize {
    #[cfg(target_feature = "avx2")]
    {
        use safe_arch::*;
        let lut_lo = m256i::from([
            10_i8, 11, 11, 11, 11, 11, 11, 11, 11, 11, 3, 1, 1, 1, 1, 5, 10, 11, 11, 11, 11, 11,
            11, 11, 11, 11, 3, 1, 1, 1, 1, 5,
        ]);
        let lut_hi = m256i::from([
            0_i8, 0, 0, 8, 1, 6, 1, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 1, 6, 1, 2, 0, 0, 0, 0,
            0, 0, 0, 0,
        ]);
        let mask_0f = set_splat_i8_m256i(0x0F);
        let zero = zeroed_m256i();

        let (chunks, _) = ptr[i..].as_chunks::<32>();
        for &chunk in chunks {
            let v = m256i::from(chunk);
            let hi = shr_imm_u16_m256i::<4>(v) & mask_0f;
            let m_lo = shuffle_av_i8z_half_m256i(lut_lo, v);
            let m_hi = shuffle_av_i8z_half_m256i(lut_hi, hi);
            let hit = m_lo & m_hi;
            let is_non_ident = cmp_eq_mask_i8_m256i(hit, zero);
            let mask = move_mask_i8_m256i(is_non_ident) as u32;
            if mask != 0 {
                return i + mask.trailing_zeros() as usize;
            }
            i += 32;
        }
    }
    while i < ptr.len() && IDENT_CONT[ptr[i] as usize] != 0 {
        i += 1;
    }
    i
}

#[derive(Clone, Copy)]
struct ParseResult {
    keyword_counts: [i64; KEYWORD_COUNT],
    total_keywords: i64,
    lines_processed: i64,
}

impl Default for ParseResult {
    fn default() -> Self {
        Self {
            keyword_counts: [0; KEYWORD_COUNT],
            total_keywords: 0,
            lines_processed: 0,
        }
    }
}

fn parse_into(content: &[u8], acc: &mut ParseResult) {
    let content_len = content.len();
    if content_len == 0 {
        return;
    }

    acc.lines_processed += count_newlines_avx2(content) as i64;

    let mut i = 0usize;
    while i < content_len {
        if TOKEN_KIND[content[i] as usize] == TOK_NONE {
            i += 1;
            if i < content_len && TOKEN_KIND[content[i] as usize] == TOK_NONE {
                i = skip_non_tokens_avx2(content, i);
            }
            if i >= content_len {
                break;
            }
        }

        match TOKEN_KIND[content[i] as usize] {
            TOK_IDENT => {
                let start = i;
                i = skip_ident_cont_avx2(content, i + 1);
                let n = i - start;
                if n.wrapping_sub(2) <= KW_MAX_LEN - 2 {
                    let h = kw_hash(content[start], content[i - 1], n);
                    let id = KW_TABLE[h];
                    if id != KW_EMPTY {
                        let idx = id as usize;
                        if KEYWORD_LENS[idx] as usize == n
                            && kw_match_len_2_to_14(
                                &content[start..i],
                                KEYWORD_NAMES[idx].as_bytes(),
                            )
                        {
                            acc.keyword_counts[idx] += 1;
                            acc.total_keywords += 1;
                        }
                    }
                }
            }
            TOK_SLASH => {
                if i + 1 < content_len {
                    let next_b = content[i + 1];
                    if next_b == b'/' {
                        i = skip_line_comment(content, i + 2);
                        continue;
                    } else if next_b == b'*' {
                        if let Some(pos) = find_star_slash(&content[i + 2..]) {
                            i += 2 + pos + 2;
                        } else {
                            i = content_len;
                        }
                        continue;
                    }
                }
                i += 1;
            }
            TOK_QUOTE => {
                i = skip_literal(content, i + 1, b'"');
            }
            TOK_APOSTROPHE => {
                if i > 0 && i + 1 < content_len {
                    let prev = content[i - 1];
                    let next = content[i + 1];
                    if IDENT_CONT[prev as usize] != 0 && IDENT_CONT[next as usize] != 0 {
                        i += 1;
                        continue;
                    }
                }
                i = skip_literal(content, i + 1, b'\'');
            }
            _ => {
                i += 1;
            }
        }
    }
}

#[inline(always)]
fn is_c_extension(name: &[u8]) -> bool {
    let len = name.len();
    if len < 2 || name[len - 2] != b'.' {
        return false;
    }
    let c = name[len - 1] | 0x20;
    c == b'c' || c == b'h'
}

const BATCH_SIZE: usize = 64;
const BATCH_BUF_SIZE: usize = 8192;

struct Batch {
    dirfd: Option<OwnedFd>,
    data: [u8; BATCH_BUF_SIZE],
    offsets: [u16; BATCH_SIZE],
    name_offsets: [u16; BATCH_SIZE],
    lens: [u16; BATCH_SIZE],
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
    fn append(&mut self, dir: &[u8], name: &[u8], too_long: &mut bool) -> bool {
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
        self.data_len = off + needed;
        self.count += 1;
        true
    }

    #[inline(always)]
    fn get_path_cstr(&self, idx: usize) -> &CStr {
        let off = self.offsets[idx] as usize;
        let len = self.lens[idx] as usize;
        CStr::from_bytes_with_nul(&self.data[off..off + len + 1]).unwrap()
    }

    #[inline(always)]
    fn get_rel_name_cstr(&self, idx: usize) -> &CStr {
        let off = self.offsets[idx] as usize;
        let name_off = self.name_offsets[idx] as usize;
        let len = self.lens[idx] as usize;
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
    mutex: Mutex<SchedulerState>,
    cond: Condvar,
}

impl WorkScheduler {
    fn new(root: &[u8]) -> Self {
        let mut dirs_arena = Vec::with_capacity(65536);
        dirs_arena.extend_from_slice(root);
        let mut dirs_spans = Vec::with_capacity(256);
        dirs_spans.push((0, root.len() as u32));

        Self {
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
        let subdirs_len = subdirs_spans.len();
        if subdirs_len > 0 {
            let base = s.dirs_arena.len() as u32;
            s.dirs_arena.extend_from_slice(subdirs_arena);
            s.dirs_spans.reserve(subdirs_len);
            for &(off, len) in subdirs_spans {
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
    parse_result: ParseResult,
    files_found: i64,
    files_processed: i64,
}

impl WorkerResult {
    fn add(&mut self, other: &Self) {
        for i in 0..KEYWORD_COUNT {
            self.parse_result.keyword_counts[i] += other.parse_result.keyword_counts[i];
        }
        self.parse_result.total_keywords += other.parse_result.total_keywords;
        self.parse_result.lines_processed += other.parse_result.lines_processed;
        self.files_found += other.files_found;
        self.files_processed += other.files_processed;
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
    res: &mut WorkerResult,
    fb: &mut Vec<u8>,
) {
    res.files_found += 1;
    if read_file_into(dirfd, path, fb) {
        res.files_processed += 1;
        parse_into(fb, &mut res.parse_result);
    }
}

fn unified_worker_proc(sched: &WorkScheduler) -> WorkerResult {
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
                        process_single_path(&dirfd, name, &mut res, &mut fb);
                    }
                } else {
                    for i in 0..b.count {
                        let path = b.get_path_cstr(i);
                        process_single_path(CWD, path, &mut res, &mut fb);
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
                        } else if ft == FileType::RegularFile && is_c_extension(name) {
                            let mut too_long = false;
                            let lb = local_batch.as_mut().unwrap();
                            if !lb.append(dir_bytes, name, &mut too_long) {
                                if too_long {
                                    process_single_path(&fd, name_cstr, &mut res, &mut fb);
                                } else {
                                    if batch_started_empty && let Ok(dup_fd) = dup(&fd) {
                                        lb.dirfd = Some(dup_fd);
                                    }
                                    let full = local_batch.take().unwrap();
                                    let mut next = sched.submit_batch_and_swap(full);
                                    batch_started_empty = true;
                                    let mut dummy = false;
                                    let ok = next.append(dir_bytes, name, &mut dummy);
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
    keyword_id: u8,
    count: i64,
    percentage: f64,
}

fn print_results(
    totals: &WorkerResult,
    processing_time: f64,
    num_workers: usize,
    cpu_count: usize,
) {
    let mut out = String::with_capacity(8192);
    let _ = writeln!(
        out,
        "Processed {} files with {} worker threads (CPU cores: {})...",
        format_number(totals.files_found),
        num_workers,
        cpu_count
    );
    let _ = writeln!(out, "C Keyword Analyzer (C Ultra - Unified Work-Stealing)");
    let _ = writeln!(
        out,
        "Files processed: {}",
        format_number(totals.files_processed)
    );
    let _ = writeln!(
        out,
        "Total lines: {}",
        format_number(totals.parse_result.lines_processed)
    );
    let _ = writeln!(
        out,
        "Total keywords: {}",
        format_number(totals.parse_result.total_keywords)
    );
    let _ = writeln!(out, "Processing time: {:.3} seconds", processing_time);

    let mut stats = Vec::with_capacity(KEYWORD_COUNT);
    let total_kw = totals.parse_result.total_keywords;

    for i in 0..KEYWORD_COUNT {
        let c = totals.parse_result.keyword_counts[i];
        if c > 0 {
            let pct = if total_kw > 0 {
                (c as f64 / total_kw as f64) * 100.0
            } else {
                0.0
            };
            stats.push(KeywordStat {
                keyword_id: i as u8,
                count: c,
                percentage: pct,
            });
        }
    }

    stats.sort_unstable_by(|a, b| match b.count.cmp(&a.count) {
        Ordering::Equal => {
            KEYWORD_NAMES[a.keyword_id as usize].cmp(KEYWORD_NAMES[b.keyword_id as usize])
        }
        ord => ord,
    });

    let _ = writeln!(out, "\nKeyword frequencies (most frequent first):");
    let _ = writeln!(out, "Rank Keyword         Count Percentage");
    let _ = writeln!(out, "---- -------         ----- ----------");

    for (i, st) in stats.iter().enumerate() {
        let _ = writeln!(
            out,
            "{:4} {:<15} {:>10} {:6.2}%",
            i + 1,
            KEYWORD_NAMES[st.keyword_id as usize],
            format_number(st.count),
            st.percentage
        );
    }

    let _ = write(stdout(), out.as_bytes());
}

fn get_cpu_count() -> usize {
    thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
}

fn main() {
    let start = Instant::now();
    let args: Vec<_> = env::args_os().collect();
    if args.len() < 2 {
        let prog = args
            .first()
            .map(|s| s.to_string_lossy())
            .unwrap_or_else(|| "c_ultra".into());
        println!(
            "Usage: {} <file_or_directory>\nAnalyzes C keyword frequencies in source files",
            prog
        );
        process::exit(1);
    }

    let target_os = &args[1];
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
        let sched = WorkScheduler::new(target_bytes);
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
        if ft != FileType::RegularFile || !is_c_extension(target_bytes) {
            eprintln!(
                "Error: {} is not a C source file",
                target_os.to_string_lossy()
            );
            process::exit(1);
        }
        num_workers = 1;
        totals.files_found = 1;
        let mut target_nul = Vec::with_capacity(target_bytes.len() + 1);
        target_nul.extend_from_slice(target_bytes);
        target_nul.push(0);
        if let Ok(target_cstr) = CStr::from_bytes_with_nul(&target_nul) {
            let mut fb = Vec::with_capacity(131072);
            if read_file_into(CWD, target_cstr, &mut fb) {
                totals.files_processed = 1;
                parse_into(&fb, &mut totals.parse_result);
            }
        }
    }

    if totals.files_found == 0 {
        println!("No C source files found");
        process::exit(1);
    }

    let processing_time = start.elapsed().as_secs_f64();
    print_results(&totals, processing_time, num_workers, cpu_count);
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustix::fs::{mkdirat, mknodat, unlinkat};
    use std::fs;

    fn find_kw_id(name: &str) -> usize {
        KEYWORD_NAMES.iter().position(|&kw| kw == name).unwrap()
    }

    #[test]
    fn test_keyword_table_and_mph() {
        assert_eq!(KEYWORD_COUNT, 59);
        let mut seen_slot = [false; 256];
        for i in 0..KEYWORD_COUNT {
            let kw = KEYWORD_NAMES[i];
            let len = kw.len();
            assert_eq!(len, KEYWORD_LENS[i] as usize);
            assert!((2..=KW_MAX_LEN).contains(&len));

            let bytes = kw.as_bytes();
            let h = kw_hash(bytes[0], bytes[len - 1], len);
            assert!(!seen_slot[h]);
            seen_slot[h] = true;
            assert_eq!(KW_TABLE[h], i as u8);
            assert!(kw_match_len_2_to_14(bytes, bytes));

            let mut res = ParseResult::default();
            parse_into(bytes, &mut res);
            assert_eq!(res.total_keywords, 1);
            assert_eq!(res.keyword_counts[i], 1);
            assert_eq!(res.lines_processed, 0);
        }

        let non_keywords = [
            "a",
            "i",
            "if_",
            "_if",
            "iff",
            "inta",
            "int_t",
            "returning",
            "autoo",
            "breaks",
            "caser",
            "chars",
            "consts",
            "continues",
            "defaults",
            "d0",
            "doubles",
            "elses",
            "enums",
            "externs",
            "floats",
            "forr",
            "gotos",
            "longs",
            "registers",
            "shorts",
            "signeds",
            "sizeofs",
            "statics",
            "structs",
            "switchs",
            "typedefs",
            "unions",
            "unsigneds",
            "voids",
            "volatiles",
            "whiles",
            "inlines",
            "restricts",
            "_Bools",
            "boolean",
            "nullptrs",
            "constexprs",
            "thread_locals",
            "static_asserts",
            "_Static_asserts",
            "class",
            "namespace",
            "template",
            "typename",
            "virtual",
            "override",
            "final",
            "private",
            "protected",
            "public",
            "friend",
            "using",
            "new",
            "delete",
            "try",
            "catch",
            "throw",
            "noexcept",
            "mutable",
            "explicit",
            "operator",
            "dynamic_cast",
            "static_cast",
            "reinterpret_cast",
            "const_cast",
            "typeid",
            "concept",
            "requires",
            "co_await",
            "co_yield",
            "co_return",
        ];
        for s in non_keywords {
            let mut res = ParseResult::default();
            parse_into(s.as_bytes(), &mut res);
            assert_eq!(res.total_keywords, 0);
        }

        for b in 0..=255u8 {
            let mut block = [b' '; 64];
            block[19] = b;
            let pos = skip_non_tokens_avx2(&block, 0);
            if TOKEN_KIND[b as usize] != TOK_NONE {
                assert_eq!(pos, 19);
            } else {
                assert_eq!(pos, block.len());
            }

            let mut block = [b'a'; 64];
            block[19] = b;
            let id_pos = skip_ident_cont_avx2(&block, 0);
            if IDENT_CONT[b as usize] == 0 {
                assert_eq!(id_pos, 19);
            } else {
                assert_eq!(id_pos, block.len());
            }
        }
    }

    #[test]
    fn test_extensions() {
        assert!(is_c_extension(b"main.c"));
        assert!(is_c_extension(b"header.h"));
        assert!(is_c_extension(b"UPPER.C"));
        assert!(is_c_extension(b"UPPER.H"));
        assert!(is_c_extension(b"a/b/c/foo.bar.c"));

        assert!(!is_c_extension(b"main.cpp"));
        assert!(!is_c_extension(b"main.cc"));
        assert!(!is_c_extension(b"main.cxx"));
        assert!(!is_c_extension(b"main.c++"));
        assert!(!is_c_extension(b"header.hpp"));
        assert!(!is_c_extension(b"header.hxx"));
        assert!(!is_c_extension(b"header.h++"));
        assert!(!is_c_extension(b" header.hh"));
        assert!(!is_c_extension(b"script.py"));
        assert!(!is_c_extension(b"Makefile"));
        assert!(!is_c_extension(b"c"));
        assert!(!is_c_extension(b""));
    }

    #[test]
    fn test_comments_and_splicing() {
        let src = concat!(
            "// single line comment with if int return while\n",
            "int a = 1;\n",
            "// spliced line comment \\\n",
            "if (a) return 0;\n",
            "// multi-spliced line comment \\\r\n",
            "while (1) \\\n",
            "for (;;)\n",
            "return a;\n",
            "/* block comment with\n",
            "   struct union enum typedef static const\n",
            "   * / ** / ***/\n",
            "char b = 2;\n",
            "/* unclosed block comment at EOF: goto switch case default"
        );

        let mut res = ParseResult::default();
        parse_into(src.as_bytes(), &mut res);

        assert_eq!(res.total_keywords, 3);
        assert_eq!(res.keyword_counts[find_kw_id("int")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("return")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("char")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("if")], 0);
        assert_eq!(res.keyword_counts[find_kw_id("while")], 0);
        assert_eq!(res.keyword_counts[find_kw_id("for")], 0);
        assert_eq!(res.keyword_counts[find_kw_id("goto")], 0);
        assert_eq!(res.lines_processed, 12);
    }

    #[test]
    fn test_literals_and_digit_separators() {
        let src = concat!(
            "const char *s1 = \"if while for return struct\";\n",
            "const char *s2 = \"escaped \\\"if return\\\" still in string\";\n",
            "const char *s3 = \"escaped backslash at end \\\\\"; if (1) {}\n",
            "char c1 = '\"'; if (1) {}\n",
            "char c2 = '\\''; while (0) {}\n",
            "char c3 = '\\\\'; for (;;) {}\n",
            "int x = 1'000'000; return x;\n",
            "unsigned y = 0xDEAD'BEEF'CAFEu; break;\n",
            "int z = 1'2'3; if (z) continue;\n"
        );

        let mut res = ParseResult::default();
        parse_into(src.as_bytes(), &mut res);

        assert_eq!(res.keyword_counts[find_kw_id("const")], 3);
        assert_eq!(res.keyword_counts[find_kw_id("char")], 6);
        assert_eq!(res.keyword_counts[find_kw_id("if")], 3);
        assert_eq!(res.keyword_counts[find_kw_id("while")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("for")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("int")], 2);
        assert_eq!(res.keyword_counts[find_kw_id("return")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("unsigned")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("break")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("continue")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("struct")], 0);
        assert_eq!(res.total_keywords, 20);
    }

    #[test]
    fn test_avx2_boundaries() {
        let len = 32 * 300 + 17;
        let mut buf = vec![b' '; len];
        let mut expected_nl = 0usize;
        for i in (0..len).step_by(7) {
            buf[i] = b'\n';
            expected_nl += 1;
        }
        assert_eq!(count_newlines_avx2(&buf), expected_nl);

        buf.fill(b' ');
        let kw = b"_Static_assert";
        let kw_len = kw.len();
        let mut placed = 0i64;
        let mut off = 25usize;
        while off + kw_len < len {
            buf[off..off + kw_len].copy_from_slice(kw);
            placed += 1;
            off += 64;
        }

        let mut res = ParseResult::default();
        parse_into(&buf, &mut res);
        assert_eq!(res.total_keywords, placed);
        assert_eq!(res.keyword_counts[find_kw_id("_Static_assert")], placed);
    }

    #[test]
    fn test_directory_traversal_and_edge_cases() {
        let tmp_dir = format!("/tmp/lfa_rust_test_{}", process::id());
        let _ = fs::remove_dir_all(&tmp_dir);
        mkdirat(CWD, tmp_dir.as_bytes(), Mode::RWXU).unwrap();

        let path_c = format!("{}/valid.c", tmp_dir);
        let path_h = format!("{}/UPPER.H", tmp_dir);
        let path_cpp = format!("{}/ignored.cpp", tmp_dir);
        let path_fifo = format!("{}/named_pipe.c", tmp_dir);
        let sub_dir = format!("{}/subdir", tmp_dir);
        let sub_c = format!("{}/subdir/nested.c", tmp_dir);

        fs::write(&path_c, "int main(void) { if (true) return 0; }\n").unwrap();
        fs::write(&path_h, "typedef struct { constexpr int x; } Foo;\n").unwrap();
        fs::write(&path_cpp, "int while for return if switch case default;\n").unwrap();
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

        let sched = WorkScheduler::new(tmp_dir.as_bytes());
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

        assert_eq!(totals.files_found, 3);
        assert_eq!(totals.files_processed, 3);
        assert_eq!(totals.parse_result.lines_processed, 3);
        assert_eq!(totals.parse_result.total_keywords, 15);
        assert_eq!(totals.parse_result.keyword_counts[find_kw_id("int")], 2);
        assert_eq!(totals.parse_result.keyword_counts[find_kw_id("void")], 2);
        assert_eq!(totals.parse_result.keyword_counts[find_kw_id("return")], 2);
        assert_eq!(totals.parse_result.keyword_counts[find_kw_id("true")], 1);
        assert_eq!(totals.parse_result.keyword_counts[find_kw_id("false")], 1);
        assert_eq!(
            totals.parse_result.keyword_counts[find_kw_id("constexpr")],
            1
        );

        unlinkat(CWD, sub_c.as_bytes(), AtFlags::empty()).unwrap();
        unlinkat(CWD, sub_dir.as_bytes(), AtFlags::REMOVEDIR).unwrap();
        unlinkat(CWD, path_fifo.as_bytes(), AtFlags::empty()).unwrap();
        unlinkat(CWD, path_cpp.as_bytes(), AtFlags::empty()).unwrap();
        unlinkat(CWD, path_h.as_bytes(), AtFlags::empty()).unwrap();
        unlinkat(CWD, path_c.as_bytes(), AtFlags::empty()).unwrap();
        unlinkat(CWD, tmp_dir.as_bytes(), AtFlags::REMOVEDIR).unwrap();
    }

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(0), "0");
        assert_eq!(format_number(999), "999");
        assert_eq!(format_number(1000), "1,000");
        assert_eq!(format_number(1234567), "1,234,567");
        assert_eq!(format_number(10000000000), "10,000,000,000");
        assert!(get_cpu_count() >= 1);
    }
}
