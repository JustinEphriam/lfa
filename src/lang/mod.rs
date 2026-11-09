pub mod c;
pub mod cpp;
pub mod rust;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LangId {
    C = 0,
    Cpp = 1,
    Rust = 2,
}

pub const LANG_COUNT: usize = 3;
pub const LANG_C_MASK: u32 = 1 << (LangId::C as u32);
pub const LANG_CPP_MASK: u32 = 1 << (LangId::Cpp as u32);
pub const LANG_RUST_MASK: u32 = 1 << (LangId::Rust as u32);
pub const LANG_ALL_MASK: u32 = (1 << LANG_COUNT) - 1;

pub const ALL_LANGS: [LangId; LANG_COUNT] = [LangId::C, LangId::Cpp, LangId::Rust];

const _: () = assert!(LANG_COUNT > 0 && LANG_COUNT <= 32);
const _: () = assert!(ALL_LANGS.len() == LANG_COUNT);

impl LangId {
    #[inline(always)]
    pub const fn name(self) -> &'static str {
        match self {
            Self::C => "C",
            Self::Cpp => "C++",
            Self::Rust => "Rust",
        }
    }

    #[inline(always)]
    pub const fn mask(self) -> u32 {
        1 << (self as u32)
    }

    #[inline(always)]
    pub fn from_flag(arg: &[u8]) -> Option<Self> {
        match arg {
            b"-c" | b"--c" | b"-C" => Some(Self::C),
            b"-cpp" | b"--cpp" | b"-CPP" | b"--CPP" => Some(Self::Cpp),
            b"-rs" | b"--rs" | b"-RS" | b"--RS" | b"-rust" | b"--rust" | b"-RUST" | b"--RUST" => {
                Some(Self::Rust)
            }
            _ => None,
        }
    }
}

#[derive(Clone, Copy)]
pub struct ParseResult<const N: usize> {
    pub keyword_counts: [i64; N],
    pub total_keywords: i64,
    pub lines_processed: i64,
}

impl<const N: usize> Default for ParseResult<N> {
    fn default() -> Self {
        Self {
            keyword_counts: [0; N],
            total_keywords: 0,
            lines_processed: 0,
        }
    }
}

impl<const N: usize> ParseResult<N> {
    #[inline(always)]
    pub fn add(&mut self, other: &Self) {
        for i in 0..N {
            self.keyword_counts[i] += other.keyword_counts[i];
        }
        self.total_keywords += other.total_keywords;
        self.lines_processed += other.lines_processed;
    }
}

#[derive(Clone, Copy, Default)]
pub struct LangStats {
    pub c: ParseResult<{ c::KEYWORD_COUNT }>,
    pub cpp: ParseResult<{ cpp::KEYWORD_COUNT }>,
    pub rust: ParseResult<{ rust::KEYWORD_COUNT }>,
}

impl LangStats {
    #[inline(always)]
    pub fn add(&mut self, other: &Self) {
        self.c.add(&other.c);
        self.cpp.add(&other.cpp);
        self.rust.add(&other.rust);
    }

    #[inline(always)]
    pub fn lines(&self, lang: LangId) -> i64 {
        match lang {
            LangId::C => self.c.lines_processed,
            LangId::Cpp => self.cpp.lines_processed,
            LangId::Rust => self.rust.lines_processed,
        }
    }

    #[inline(always)]
    pub fn total_keywords(&self, lang: LangId) -> i64 {
        match lang {
            LangId::C => self.c.total_keywords,
            LangId::Cpp => self.cpp.total_keywords,
            LangId::Rust => self.rust.total_keywords,
        }
    }

    #[inline(always)]
    pub fn keyword_names(lang: LangId) -> &'static [&'static str] {
        match lang {
            LangId::C => &c::KEYWORD_NAMES,
            LangId::Cpp => &cpp::KEYWORD_NAMES,
            LangId::Rust => &rust::KEYWORD_NAMES,
        }
    }

    #[inline(always)]
    pub fn keyword_counts(&self, lang: LangId) -> &[i64] {
        match lang {
            LangId::C => &self.c.keyword_counts,
            LangId::Cpp => &self.cpp.keyword_counts,
            LangId::Rust => &self.rust.keyword_counts,
        }
    }
}

#[inline(always)]
pub fn detect_language(name: &[u8], enabled_mask: u32) -> Option<LangId> {
    if (enabled_mask & LANG_C_MASK) != 0 && c::is_c_extension(name) {
        return Some(LangId::C);
    }
    if (enabled_mask & LANG_RUST_MASK) != 0 && rust::is_rust_extension(name) {
        return Some(LangId::Rust);
    }
    if (enabled_mask & LANG_CPP_MASK) != 0 && (cpp::is_cpp_extension(name) || c::is_h_extension(name)) {
        return Some(LangId::Cpp);
    }
    None
}

#[inline(always)]
pub fn parse_into(lang: LangId, content: &[u8], stats: &mut LangStats) {
    match lang {
        LangId::C => c::parse_into(content, &mut stats.c),
        LangId::Cpp => cpp::parse_into(content, &mut stats.cpp),
        LangId::Rust => rust::parse_into(content, &mut stats.rust),
    }
}

#[inline(always)]
pub fn count_newlines_avx2(bytes: &[u8]) -> usize {
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
pub fn find_byte_avx2(slice: &[u8], needle: u8) -> Option<usize> {
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
pub fn find_star_slash(slice: &[u8]) -> Option<usize> {
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
pub fn skip_line_comment(content: &[u8], start_i: usize) -> usize {
    debug_assert!(start_i <= content.len());
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
pub fn skip_literal(content: &[u8], start_i: usize, quote: u8) -> usize {
    debug_assert!(start_i <= content.len());
    debug_assert!(quote == b'"' || quote == b'\'');
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_newlines_saturation_and_boundaries() {
        assert_eq!(count_newlines_avx2(b""), 0);
        assert_eq!(count_newlines_avx2(b"a"), 0);
        assert_eq!(count_newlines_avx2(b"\n"), 1);

        for len in [1, 31, 32, 33, 63, 64, 65, 8159, 8160, 8161, 17000] {
            let all_nl = vec![b'\n'; len];
            assert_eq!(count_newlines_avx2(&all_nl), len);

            let no_nl = vec![b'x'; len];
            assert_eq!(count_newlines_avx2(&no_nl), 0);
        }
    }

    #[test]
    fn test_find_byte_and_star_slash_boundaries() {
        assert_eq!(find_byte_avx2(b"", b'x'), None);
        assert_eq!(find_star_slash(b""), None);
        assert_eq!(find_star_slash(b"*"), None);
        assert_eq!(find_star_slash(b"/"), None);
        assert_eq!(find_star_slash(b"/*"), None);

        for total_len in [2, 31, 32, 33, 63, 64, 65, 96, 100] {
            for pos in 0..total_len {
                let mut buf = vec![b'.'; total_len];
                buf[pos] = b'!';
                assert_eq!(find_byte_avx2(&buf, b'!'), Some(pos));
                assert_eq!(find_byte_avx2(&buf, b'?'), None);
            }

            for pos in 0..total_len - 1 {
                let mut buf = vec![b'.'; total_len];
                buf[pos] = b'*';
                buf[pos + 1] = b'/';
                assert_eq!(find_star_slash(&buf), Some(pos));
            }
        }
    }

    #[test]
    fn test_skip_comment_and_literal_edge_cases() {
        assert_eq!(skip_line_comment(b"no newline at all", 0), 17);
        assert_eq!(skip_line_comment(b"trailing backslash \\", 0), 20);
        assert_eq!(skip_line_comment(b"spliced \\\nstill comment\nnext", 0), 24);
        assert_eq!(
            skip_line_comment(b"spliced crlf \\\r\nstill comment\r\nnext", 0),
            31
        );

        assert_eq!(skip_literal(b"unclosed string", 0, b'"'), 15);
        assert_eq!(skip_literal(b"trailing escape \\", 0, b'"'), 17);
        assert_eq!(skip_literal(b"\\\\\"rest", 0, b'"'), 3);
        assert_eq!(skip_literal(b"\\\\\\\"\"rest", 0, b'"'), 5);

        for offset in 28..36 {
            let mut s = vec![b'a'; 80];
            s[offset] = b'\\';
            s[offset + 1] = b'"';
            s[offset + 10] = b'"';
            assert_eq!(skip_literal(&s, 0, b'"'), offset + 11);
        }
    }

    #[test]
    fn test_detect_language_and_header_routing() {
        assert_eq!(detect_language(b"foo.c", LANG_ALL_MASK), Some(LangId::C));
        assert_eq!(detect_language(b"foo.h", LANG_ALL_MASK), Some(LangId::C));
        assert_eq!(detect_language(b"foo.H", LANG_ALL_MASK), Some(LangId::C));
        assert_eq!(
            detect_language(b"foo.cpp", LANG_ALL_MASK),
            Some(LangId::Cpp)
        );
        assert_eq!(
            detect_language(b"foo.hpp", LANG_ALL_MASK),
            Some(LangId::Cpp)
        );
        assert_eq!(
            detect_language(b"foo.rs", LANG_ALL_MASK),
            Some(LangId::Rust)
        );
        assert_eq!(detect_language(b"foo.py", LANG_ALL_MASK), None);

        assert_eq!(detect_language(b"foo.c", LANG_C_MASK), Some(LangId::C));
        assert_eq!(detect_language(b"foo.h", LANG_C_MASK), Some(LangId::C));
        assert_eq!(detect_language(b"foo.cpp", LANG_C_MASK), None);
        assert_eq!(detect_language(b"foo.hpp", LANG_C_MASK), None);

        assert_eq!(detect_language(b"foo.c", LANG_CPP_MASK), None);
        assert_eq!(detect_language(b"foo.h", LANG_CPP_MASK), Some(LangId::Cpp));
        assert_eq!(detect_language(b"foo.H", LANG_CPP_MASK), Some(LangId::Cpp));
        assert_eq!(
            detect_language(b"foo.cpp", LANG_CPP_MASK),
            Some(LangId::Cpp)
        );
        assert_eq!(
            detect_language(b"foo.hpp", LANG_CPP_MASK),
            Some(LangId::Cpp)
        );

        assert_eq!(detect_language(b"foo.c", 0), None);
        assert_eq!(detect_language(b"foo.h", 0), None);
        assert_eq!(detect_language(b"foo.cpp", 0), None);
    }
}
