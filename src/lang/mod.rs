pub mod c;
pub mod cpp;
pub mod go;
pub mod javascript;
pub mod odin;
pub mod perl;
pub mod python;
pub mod rust;
pub mod typescript;
pub mod zig;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LangId {
    C = 0,
    Cpp = 1,
    Rust = 2,
    Python = 3,
    Perl = 4,
    Go = 5,
    Zig = 6,
    Odin = 7,
    JavaScript = 8,
    TypeScript = 9,
}

pub const LANG_COUNT: usize = 10;
pub const LANG_C_MASK: u32 = 1 << (LangId::C as u32);
pub const LANG_CPP_MASK: u32 = 1 << (LangId::Cpp as u32);
pub const LANG_RUST_MASK: u32 = 1 << (LangId::Rust as u32);
pub const LANG_PYTHON_MASK: u32 = 1 << (LangId::Python as u32);
pub const LANG_PERL_MASK: u32 = 1 << (LangId::Perl as u32);
pub const LANG_GO_MASK: u32 = 1 << (LangId::Go as u32);
pub const LANG_ZIG_MASK: u32 = 1 << (LangId::Zig as u32);
pub const LANG_ODIN_MASK: u32 = 1 << (LangId::Odin as u32);
pub const LANG_JAVASCRIPT_MASK: u32 = 1 << (LangId::JavaScript as u32);
pub const LANG_TYPESCRIPT_MASK: u32 = 1 << (LangId::TypeScript as u32);
pub const LANG_ALL_MASK: u32 = (1 << LANG_COUNT) - 1;

pub const ALL_LANGS: [LangId; LANG_COUNT] = [
    LangId::C,
    LangId::Cpp,
    LangId::Rust,
    LangId::Python,
    LangId::Perl,
    LangId::Go,
    LangId::Zig,
    LangId::Odin,
    LangId::JavaScript,
    LangId::TypeScript,
];

const _: () = assert!(LANG_COUNT > 0 && LANG_COUNT <= 32);
const _: () = assert!(ALL_LANGS.len() == LANG_COUNT);

impl LangId {
    #[inline(always)]
    pub const fn name(self) -> &'static str {
        match self {
            Self::C => "C",
            Self::Cpp => "C++",
            Self::Rust => "Rust",
            Self::Python => "Python",
            Self::Perl => "Perl",
            Self::Go => "Go",
            Self::Zig => "Zig",
            Self::Odin => "Odin",
            Self::JavaScript => "JavaScript",
            Self::TypeScript => "TypeScript",
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
            b"-py" | b"--py" | b"-PY" | b"--PY" | b"-python" | b"--python" | b"-PYTHON"
            | b"--PYTHON" => Some(Self::Python),
            b"-pl" | b"--pl" | b"-PL" | b"--PL" | b"-perl" | b"--perl" | b"-PERL" | b"--PERL" => {
                Some(Self::Perl)
            }
            b"-go" | b"--go" | b"-GO" | b"--GO" | b"-golang" | b"--golang" | b"-GOLANG"
            | b"--GOLANG" => Some(Self::Go),
            b"-zig" | b"--zig" | b"-ZIG" | b"--ZIG" => Some(Self::Zig),
            b"-odin" | b"--odin" | b"-ODIN" | b"--ODIN" => Some(Self::Odin),
            b"-js" | b"--js" | b"-JS" | b"--JS" | b"-javascript" | b"--javascript"
            | b"-JAVASCRIPT" | b"--JAVASCRIPT" => Some(Self::JavaScript),
            b"-ts" | b"--ts" | b"-TS" | b"--TS" | b"-typescript" | b"--typescript"
            | b"-TYPESCRIPT" | b"--TYPESCRIPT" => Some(Self::TypeScript),
            _ => None,
        }
    }
}

#[derive(Clone, Copy)]
pub struct ParseResult<const N: usize> {
    pub keyword_counts: [i64; N],
    pub total_keywords: i64,
    pub lines_processed: i64,
    pub code_lines: i64,
    pub comment_lines: i64,
    pub blank_lines: i64,
}

impl<const N: usize> Default for ParseResult<N> {
    fn default() -> Self {
        Self {
            keyword_counts: [0; N],
            total_keywords: 0,
            lines_processed: 0,
            code_lines: 0,
            comment_lines: 0,
            blank_lines: 0,
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
        self.code_lines += other.code_lines;
        self.comment_lines += other.comment_lines;
        self.blank_lines += other.blank_lines;
    }
}

#[derive(Clone, Copy, Default)]
pub struct LangStats {
    pub c: ParseResult<{ c::KEYWORD_COUNT }>,
    pub cpp: ParseResult<{ cpp::KEYWORD_COUNT }>,
    pub rust: ParseResult<{ rust::KEYWORD_COUNT }>,
    pub python: ParseResult<{ python::KEYWORD_COUNT }>,
    pub perl: ParseResult<{ perl::KEYWORD_COUNT }>,
    pub go: ParseResult<{ go::KEYWORD_COUNT }>,
    pub zig: ParseResult<{ zig::KEYWORD_COUNT }>,
    pub odin: ParseResult<{ odin::KEYWORD_COUNT }>,
    pub javascript: ParseResult<{ javascript::KEYWORD_COUNT }>,
    pub typescript: ParseResult<{ typescript::KEYWORD_COUNT }>,
}

impl LangStats {
    #[inline(always)]
    pub fn add(&mut self, other: &Self) {
        self.c.add(&other.c);
        self.cpp.add(&other.cpp);
        self.rust.add(&other.rust);
        self.python.add(&other.python);
        self.perl.add(&other.perl);
        self.go.add(&other.go);
        self.zig.add(&other.zig);
        self.odin.add(&other.odin);
        self.javascript.add(&other.javascript);
        self.typescript.add(&other.typescript);
    }

    #[inline(always)]
    pub fn lines(&self, lang: LangId) -> i64 {
        match lang {
            LangId::C => self.c.lines_processed,
            LangId::Cpp => self.cpp.lines_processed,
            LangId::Rust => self.rust.lines_processed,
            LangId::Python => self.python.lines_processed,
            LangId::Perl => self.perl.lines_processed,
            LangId::Go => self.go.lines_processed,
            LangId::Zig => self.zig.lines_processed,
            LangId::Odin => self.odin.lines_processed,
            LangId::JavaScript => self.javascript.lines_processed,
            LangId::TypeScript => self.typescript.lines_processed,
        }
    }

    #[inline(always)]
    pub fn code_lines(&self, lang: LangId) -> i64 {
        match lang {
            LangId::C => self.c.code_lines,
            LangId::Cpp => self.cpp.code_lines,
            LangId::Rust => self.rust.code_lines,
            LangId::Python => self.python.code_lines,
            LangId::Perl => self.perl.code_lines,
            LangId::Go => self.go.code_lines,
            LangId::Zig => self.zig.code_lines,
            LangId::Odin => self.odin.code_lines,
            LangId::JavaScript => self.javascript.code_lines,
            LangId::TypeScript => self.typescript.code_lines,
        }
    }

    #[inline(always)]
    pub fn comment_lines(&self, lang: LangId) -> i64 {
        match lang {
            LangId::C => self.c.comment_lines,
            LangId::Cpp => self.cpp.comment_lines,
            LangId::Rust => self.rust.comment_lines,
            LangId::Python => self.python.comment_lines,
            LangId::Perl => self.perl.comment_lines,
            LangId::Go => self.go.comment_lines,
            LangId::Zig => self.zig.comment_lines,
            LangId::Odin => self.odin.comment_lines,
            LangId::JavaScript => self.javascript.comment_lines,
            LangId::TypeScript => self.typescript.comment_lines,
        }
    }

    #[inline(always)]
    pub fn blank_lines(&self, lang: LangId) -> i64 {
        match lang {
            LangId::C => self.c.blank_lines,
            LangId::Cpp => self.cpp.blank_lines,
            LangId::Rust => self.rust.blank_lines,
            LangId::Python => self.python.blank_lines,
            LangId::Perl => self.perl.blank_lines,
            LangId::Go => self.go.blank_lines,
            LangId::Zig => self.zig.blank_lines,
            LangId::Odin => self.odin.blank_lines,
            LangId::JavaScript => self.javascript.blank_lines,
            LangId::TypeScript => self.typescript.blank_lines,
        }
    }

    #[inline(always)]
    pub fn total_keywords(&self, lang: LangId) -> i64 {
        match lang {
            LangId::C => self.c.total_keywords,
            LangId::Cpp => self.cpp.total_keywords,
            LangId::Rust => self.rust.total_keywords,
            LangId::Python => self.python.total_keywords,
            LangId::Perl => self.perl.total_keywords,
            LangId::Go => self.go.total_keywords,
            LangId::Zig => self.zig.total_keywords,
            LangId::Odin => self.odin.total_keywords,
            LangId::JavaScript => self.javascript.total_keywords,
            LangId::TypeScript => self.typescript.total_keywords,
        }
    }

    #[inline(always)]
    pub fn keyword_names(lang: LangId) -> &'static [&'static str] {
        match lang {
            LangId::C => &c::KEYWORD_NAMES,
            LangId::Cpp => &cpp::KEYWORD_NAMES,
            LangId::Rust => &rust::KEYWORD_NAMES,
            LangId::Python => &python::KEYWORD_NAMES,
            LangId::Perl => &perl::KEYWORD_NAMES,
            LangId::Go => &go::KEYWORD_NAMES,
            LangId::Zig => &zig::KEYWORD_NAMES,
            LangId::Odin => &odin::KEYWORD_NAMES,
            LangId::JavaScript => &javascript::KEYWORD_NAMES,
            LangId::TypeScript => &typescript::KEYWORD_NAMES,
        }
    }

    #[inline(always)]
    pub fn keyword_counts(&self, lang: LangId) -> &[i64] {
        match lang {
            LangId::C => &self.c.keyword_counts,
            LangId::Cpp => &self.cpp.keyword_counts,
            LangId::Rust => &self.rust.keyword_counts,
            LangId::Python => &self.python.keyword_counts,
            LangId::Perl => &self.perl.keyword_counts,
            LangId::Go => &self.go.keyword_counts,
            LangId::Zig => &self.zig.keyword_counts,
            LangId::Odin => &self.odin.keyword_counts,
            LangId::JavaScript => &self.javascript.keyword_counts,
            LangId::TypeScript => &self.typescript.keyword_counts,
        }
    }
}

#[inline(always)]
pub fn detect_language(name: &[u8], enabled_mask: u32) -> Option<LangId> {
    let len = name.len();
    if len < 2 {
        return None;
    }
    if name[len - 2] == b'.' {
        match name[len - 1] | 0x20 {
            b'c' if (enabled_mask & LANG_C_MASK) != 0 => return Some(LangId::C),
            b'h' => {
                if (enabled_mask & LANG_C_MASK) != 0 {
                    return Some(LangId::C);
                }
                if (enabled_mask & LANG_CPP_MASK) != 0 {
                    return Some(LangId::Cpp);
                }
            }
            b't' if (enabled_mask & LANG_PERL_MASK) != 0 && len >= 3 => {
                let prev = name[len - 3];
                if prev != b'/' && prev != b'\\' {
                    return Some(LangId::Perl);
                }
            }
            _ => {}
        }
        return None;
    }
    if len >= 3 && name[len - 3] == b'.' {
        let ext = [name[len - 2] | 0x20, name[len - 1] | 0x20];
        match &ext {
            b"rs" if (enabled_mask & LANG_RUST_MASK) != 0 && len >= 4 => {
                return Some(LangId::Rust);
            }
            b"cc" | b"hh" if (enabled_mask & LANG_CPP_MASK) != 0 => {
                return Some(LangId::Cpp);
            }
            b"py" if (enabled_mask & LANG_PYTHON_MASK) != 0 => {
                return Some(LangId::Python);
            }
            b"pl" | b"pm" if (enabled_mask & LANG_PERL_MASK) != 0 && len >= 4 => {
                let prev = name[len - 4];
                if prev != b'/' && prev != b'\\' {
                    return Some(LangId::Perl);
                }
            }
            b"go" if (enabled_mask & LANG_GO_MASK) != 0 && len >= 4 => {
                return Some(LangId::Go);
            }
            b"js" if (enabled_mask & LANG_JAVASCRIPT_MASK) != 0 && len >= 4 => {
                let prev = name[len - 4];
                if prev != b'/' && prev != b'\\' {
                    return Some(LangId::JavaScript);
                }
            }
            b"ts" if (enabled_mask & LANG_TYPESCRIPT_MASK) != 0 && len >= 4 => {
                let prev = name[len - 4];
                if prev != b'/' && prev != b'\\' {
                    return Some(LangId::TypeScript);
                }
            }
            _ => {}
        }
        return None;
    }
    if len >= 4 && name[len - 4] == b'.' {
        let ext = [
            name[len - 3] | 0x20,
            name[len - 2] | 0x20,
            name[len - 1] | 0x20,
        ];
        match &ext {
            b"cpp" | b"cxx" | b"c++" | b"hpp" | b"hxx" | b"h++" | b"ipp" | b"inl" | b"ixx"
                if (enabled_mask & LANG_CPP_MASK) != 0 =>
            {
                return Some(LangId::Cpp);
            }
            b"pyi" | b"pyw" if (enabled_mask & LANG_PYTHON_MASK) != 0 => {
                return Some(LangId::Python);
            }
            b"zig" | b"zon" if (enabled_mask & LANG_ZIG_MASK) != 0 && len >= 5 => {
                return Some(LangId::Zig);
            }
            b"mjs" | b"cjs" | b"jsx" if (enabled_mask & LANG_JAVASCRIPT_MASK) != 0 && len >= 5 => {
                return Some(LangId::JavaScript);
            }
            b"mts" | b"cts" | b"tsx" if (enabled_mask & LANG_TYPESCRIPT_MASK) != 0 && len >= 5 => {
                return Some(LangId::TypeScript);
            }
            _ => {}
        }
        return None;
    }
    if len >= 5 && name[len - 5] == b'.' {
        let ext = [
            name[len - 4] | 0x20,
            name[len - 3] | 0x20,
            name[len - 2] | 0x20,
            name[len - 1] | 0x20,
        ];
        match &ext {
            b"odin" if (enabled_mask & LANG_ODIN_MASK) != 0 && len >= 6 => {
                return Some(LangId::Odin);
            }
            b"cppm" if (enabled_mask & LANG_CPP_MASK) != 0 => {
                return Some(LangId::Cpp);
            }
            _ => {}
        }
    }
    None
}

pub const IDENT_CONT: [u8; 256] = {
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
pub fn skip_ident_cont_avx2(ptr: &[u8], mut i: usize) -> usize {
    debug_assert!(i <= ptr.len());
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

#[inline(always)]
pub fn kw_match_2_to_16(cand: &[u8], kw: &[u8]) -> bool {
    let n = cand.len();
    if n != kw.len() || !(2..=16).contains(&n) {
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
pub fn skip_nested_block_comment(content: &[u8], start_i: usize) -> usize {
    let content_len = content.len();
    let Some(first_close) = find_star_slash(&content[start_i..]) else {
        return content_len;
    };
    let mut i = start_i;
    let first_end = start_i + first_close;
    let mut has_nested = false;
    let mut k = start_i;
    while k + 1 < first_end {
        let Some(pos) = find_byte_avx2(&content[k..first_end - 1], b'/') else {
            break;
        };
        if content[k + pos + 1] == b'*' {
            has_nested = true;
            break;
        }
        k += pos + 1;
    }
    if !has_nested {
        return first_end + 2;
    }

    let mut depth = 1usize;
    while i + 1 < content_len {
        let b0 = content[i];
        let b1 = content[i + 1];
        if b0 == b'/' && b1 == b'*' {
            depth += 1;
            i += 2;
        } else if b0 == b'*' && b1 == b'/' {
            depth -= 1;
            i += 2;
            if depth == 0 {
                return i;
            }
        } else {
            i += 1;
        }
    }
    content_len
}

#[inline(always)]
pub fn skip_triple_literal(content: &[u8], start_i: usize, quote: u8) -> usize {
    debug_assert!(start_i <= content.len());
    let content_len = content.len();
    let mut i = start_i;
    while i + 2 < content_len {
        let Some(pos) = find_byte_avx2(&content[i..content_len - 2], quote) else {
            return content_len;
        };
        let q_idx = i + pos;
        if content[q_idx + 1] == quote && content[q_idx + 2] == quote {
            let mut escapes = 0usize;
            let mut j = q_idx;
            while j > start_i && content[j - 1] == b'\\' {
                escapes += 1;
                j -= 1;
            }
            if (escapes & 1) == 0 {
                return q_idx + 3;
            }
        }
        i = q_idx + 1;
    }
    content_len
}

#[inline(always)]
pub fn parse_into(lang: LangId, content: &[u8], stats: &mut LangStats) {
    match lang {
        LangId::C => c::parse_into(content, &mut stats.c),
        LangId::Cpp => cpp::parse_into(content, &mut stats.cpp),
        LangId::Rust => rust::parse_into(content, &mut stats.rust),
        LangId::Python => python::parse_into(content, &mut stats.python),
        LangId::Perl => perl::parse_into(content, &mut stats.perl),
        LangId::Go => go::parse_into(content, &mut stats.go),
        LangId::Zig => zig::parse_into(content, &mut stats.zig),
        LangId::Odin => odin::parse_into(content, &mut stats.odin),
        LangId::JavaScript => javascript::parse_into(content, &mut stats.javascript),
        LangId::TypeScript => typescript::parse_into(content, &mut stats.typescript),
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

#[allow(dead_code)]
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
    debug_assert!(quote == b'"' || quote == b'\'' || quote == b'`');
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
        assert_eq!(
            detect_language(b"foo.py", LANG_ALL_MASK),
            Some(LangId::Python)
        );
        assert_eq!(
            detect_language(b"foo.pl", LANG_ALL_MASK),
            Some(LangId::Perl)
        );
        assert_eq!(
            detect_language(b"foo.pm", LANG_ALL_MASK),
            Some(LangId::Perl)
        );
        assert_eq!(
            detect_language(b"foo.t", LANG_ALL_MASK),
            Some(LangId::Perl)
        );
        assert_eq!(
            detect_language(b"foo.go", LANG_ALL_MASK),
            Some(LangId::Go)
        );
        assert_eq!(
            detect_language(b"foo.zig", LANG_ALL_MASK),
            Some(LangId::Zig)
        );
        assert_eq!(
            detect_language(b"foo.zon", LANG_ALL_MASK),
            Some(LangId::Zig)
        );
        assert_eq!(
            detect_language(b"foo.odin", LANG_ALL_MASK),
            Some(LangId::Odin)
        );
        assert_eq!(
            detect_language(b"foo.js", LANG_ALL_MASK),
            Some(LangId::JavaScript)
        );
        assert_eq!(
            detect_language(b"foo.mjs", LANG_ALL_MASK),
            Some(LangId::JavaScript)
        );
        assert_eq!(
            detect_language(b"foo.cjs", LANG_ALL_MASK),
            Some(LangId::JavaScript)
        );
        assert_eq!(
            detect_language(b"foo.jsx", LANG_ALL_MASK),
            Some(LangId::JavaScript)
        );
        assert_eq!(
            detect_language(b"foo.ts", LANG_ALL_MASK),
            Some(LangId::TypeScript)
        );
        assert_eq!(
            detect_language(b"foo.mts", LANG_ALL_MASK),
            Some(LangId::TypeScript)
        );
        assert_eq!(
            detect_language(b"foo.cts", LANG_ALL_MASK),
            Some(LangId::TypeScript)
        );
        assert_eq!(
            detect_language(b"foo.tsx", LANG_ALL_MASK),
            Some(LangId::TypeScript)
        );
        assert_eq!(detect_language(b"foo.rb", LANG_ALL_MASK), None);

        assert_eq!(detect_language(b"foo.c", LANG_C_MASK), Some(LangId::C));
        assert_eq!(detect_language(b"foo.h", LANG_C_MASK), Some(LangId::C));
        assert_eq!(detect_language(b"foo.cpp", LANG_C_MASK), None);
        assert_eq!(detect_language(b"foo.hpp", LANG_C_MASK), None);
        assert_eq!(detect_language(b"foo.py", LANG_C_MASK), None);
        assert_eq!(detect_language(b"foo.pl", LANG_C_MASK), None);
        assert_eq!(detect_language(b"foo.go", LANG_C_MASK), None);

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
        assert_eq!(detect_language(b"foo.py", LANG_CPP_MASK), None);
        assert_eq!(detect_language(b"foo.pl", LANG_CPP_MASK), None);
        assert_eq!(detect_language(b"foo.go", LANG_CPP_MASK), None);

        assert_eq!(detect_language(b"foo.py", LANG_PYTHON_MASK), Some(LangId::Python));
        assert_eq!(detect_language(b"foo.c", LANG_PYTHON_MASK), None);

        assert_eq!(detect_language(b"foo.pl", LANG_PERL_MASK), Some(LangId::Perl));
        assert_eq!(detect_language(b"foo.pm", LANG_PERL_MASK), Some(LangId::Perl));
        assert_eq!(detect_language(b"foo.t", LANG_PERL_MASK), Some(LangId::Perl));
        assert_eq!(detect_language(b"foo.c", LANG_PERL_MASK), None);

        assert_eq!(detect_language(b"foo.go", LANG_GO_MASK), Some(LangId::Go));
        assert_eq!(detect_language(b"foo.c", LANG_GO_MASK), None);

        assert_eq!(detect_language(b"foo.zig", LANG_ZIG_MASK), Some(LangId::Zig));
        assert_eq!(detect_language(b"foo.zon", LANG_ZIG_MASK), Some(LangId::Zig));
        assert_eq!(detect_language(b"foo.c", LANG_ZIG_MASK), None);

        assert_eq!(detect_language(b"foo.odin", LANG_ODIN_MASK), Some(LangId::Odin));
        assert_eq!(detect_language(b"foo.c", LANG_ODIN_MASK), None);

        assert_eq!(detect_language(b"foo.c", 0), None);
        assert_eq!(detect_language(b"foo.h", 0), None);
        assert_eq!(detect_language(b"foo.cpp", 0), None);
    }
}
