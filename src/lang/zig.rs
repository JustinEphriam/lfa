use super::{count_newlines_avx2, find_byte_avx2, skip_literal, ParseResult};

pub const KEYWORD_COUNT: usize = 46;
const KW_SLOT_MASK: usize = 255;
const KW_MAX_LEN: usize = 13;
const KW_EMPTY: u8 = 255;

pub const KEYWORD_NAMES: [&str; KEYWORD_COUNT] = [
    "addrspace",
    "align",
    "allowzero",
    "and",
    "anyframe",
    "anytype",
    "asm",
    "break",
    "callconv",
    "catch",
    "comptime",
    "const",
    "continue",
    "defer",
    "else",
    "enum",
    "errdefer",
    "error",
    "export",
    "extern",
    "fn",
    "for",
    "if",
    "inline",
    "linksection",
    "noalias",
    "noinline",
    "nosuspend",
    "opaque",
    "or",
    "orelse",
    "packed",
    "pub",
    "resume",
    "return",
    "struct",
    "suspend",
    "switch",
    "test",
    "threadlocal",
    "try",
    "union",
    "unreachable",
    "var",
    "volatile",
    "while",
];

pub const KEYWORD_LENS: [u8; KEYWORD_COUNT] = [
    9, 5, 9, 3, 8, 7, 3, 5, 8, 5, 8, 5, 8, 5, 4, 4, 8, 5, 6, 6, 2, 3, 2, 6, 11, 7, 8, 9, 6, 2, 6,
    6, 3, 6, 6, 6, 7, 6, 4, 11, 3, 5, 11, 3, 8, 5,
];

const _: () = assert!(KEYWORD_NAMES.len() == KEYWORD_COUNT);
const _: () = assert!(KEYWORD_LENS.len() == KEYWORD_COUNT);

const TOK_WS: u8 = 0;
const TOK_NL: u8 = 1;
const TOK_CODE: u8 = 2;
const TOK_IDENT: u8 = 3;
const TOK_SLASH: u8 = 4;
const TOK_QUOTE: u8 = 5;
const TOK_APOSTROPHE: u8 = 6;
const TOK_BACKSLASH: u8 = 7;

pub const TOKEN_KIND: [u8; 256] = [
    TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE,
    TOK_CODE, TOK_WS, TOK_NL, TOK_CODE, TOK_CODE, TOK_WS, TOK_CODE, TOK_CODE,
    TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE,
    TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE,
    TOK_WS, TOK_CODE, TOK_QUOTE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_APOSTROPHE,
    TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_SLASH,
    TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT,
    TOK_IDENT, TOK_IDENT, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE,
    TOK_CODE, TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT,
    TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT,
    TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT,
    TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_CODE, TOK_BACKSLASH, TOK_CODE, TOK_CODE, TOK_IDENT,
    TOK_CODE, TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT,
    TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT,
    TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT,
    TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE,
    TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE,
    TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE,
    TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE,
    TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE,
    TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE,
    TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE,
    TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE,
    TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE,
    TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE,
    TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE,
    TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE,
    TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE,
    TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE,
    TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE,
    TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE,
    TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE,
];

pub const IDENT_CONT: [u8; 256] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0,
    0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 0, 0, 0, 0, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

pub const KW_SLOT: [u8; 256] = [
    255, 255, 255, 23, 255, 255, 17, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 13, 40, 255, 31, 255, 16, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 25, 255, 255,
    255, 255, 255, 12, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 9, 255, 30, 255, 255, 32, 255, 255, 36, 255, 29, 24, 255, 33,
    7, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 8, 255,
    255, 43, 44, 255, 22, 11, 35, 255, 255, 255, 255, 28, 255, 255, 42, 255,
    255, 255, 255, 255, 255, 15, 255, 255, 255, 255, 5, 27, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 2, 255, 255,
    255, 255, 255, 255, 10, 255, 255, 255, 255, 255, 21, 255, 37, 255, 255, 255,
    255, 19, 255, 255, 255, 38, 255, 255, 14, 255, 6, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 26, 34,
    255, 1, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 41, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 18, 255, 255,
    255, 4, 255, 255, 255, 255, 255, 255, 255, 45, 20, 255, 255, 3, 39, 255,
];

#[inline(always)]
pub fn kw_hash(first: u8, mid: u8, last: u8, len: usize) -> usize {
    let val = (first as usize) * 1204
        + (mid as usize) * 3183
        + (last as usize) * 1459
        + len * 3884;
    (val ^ (val >> 1)) & KW_SLOT_MASK
}

#[inline(always)]
fn skip_non_tokens_avx2(ptr: &[u8], mut i: usize) -> usize {
    debug_assert!(i <= ptr.len());
    #[cfg(target_feature = "avx2")]
    {
        use safe_arch::*;
        let lut_lo = m256i::from([
            3_i8, 3, 11, 11, 3, 3, 3, 11, 3, 3, 19, 1, 1, 1, 1, 13, 3, 3, 11, 11, 3, 3, 3, 11, 3,
            3, 19, 1, 1, 1, 1, 13,
        ]);
        let lut_hi = m256i::from([
            16_i8, 0, 8, 0, 1, 6, 1, 2, 0, 0, 0, 0, 0, 0, 0, 0, 16, 0, 8, 0, 1, 6, 1, 2, 0, 0, 0,
            0, 0, 0, 0, 0,
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
    while i < ptr.len() && TOKEN_KIND[ptr[i] as usize] <= TOK_CODE && ptr[i] != b'\n' {
        i += 1;
    }
    i
}

#[inline(always)]
fn skip_ident_cont_avx2(ptr: &[u8], mut i: usize) -> usize {
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
pub fn is_zig_extension(name: &[u8]) -> bool {
    let len = name.len();
    if len >= 5 && name[len - 4] == b'.' {
        let z = name[len - 3] | 0x20;
        let i = name[len - 2] | 0x20;
        let g = name[len - 1] | 0x20;
        if z == b'z' && i == b'i' && g == b'g' {
            return true;
        }
        let o = name[len - 2] | 0x20;
        let n = name[len - 1] | 0x20;
        if z == b'z' && o == b'o' && n == b'n' {
            return true;
        }
    }
    false
}

pub fn parse_into(content: &[u8], acc: &mut ParseResult<KEYWORD_COUNT>) {
    let content_len = content.len();
    if content_len == 0 {
        return;
    }

    const STATE_NONE: u8 = 0;
    const STATE_CODE: u8 = 1;
    const STATE_COMMENT: u8 = 2;

    let mut line_state = STATE_NONE;
    let mut i = 0usize;
    while i < content_len {
        if line_state == STATE_NONE {
            while i < content_len
                && (content[i] == b' ' || content[i] == b'\t' || content[i] == b'\r')
            {
                i += 1;
            }
            if i >= content_len {
                break;
            }
            if content[i] == b'\n' {
                acc.blank_lines += 1;
                acc.lines_processed += 1;
                i += 1;
                continue;
            }
        }

        if TOKEN_KIND[content[i] as usize] <= TOK_CODE && content[i] != b'\n' {
            line_state |= STATE_CODE;
            i += 1;
            if i < content_len
                && TOKEN_KIND[content[i] as usize] <= TOK_CODE
                && content[i] != b'\n'
            {
                i = skip_non_tokens_avx2(content, i);
            }
            if i >= content_len {
                break;
            }
        }

        let b = content[i];
        let kind = TOKEN_KIND[b as usize];
        match kind {
            TOK_NL => {
                match line_state {
                    STATE_NONE => acc.blank_lines += 1,
                    STATE_COMMENT => acc.comment_lines += 1,
                    _ => acc.code_lines += 1,
                }
                acc.lines_processed += 1;
                line_state = STATE_NONE;
                i += 1;
            }
            TOK_IDENT => {
                line_state |= STATE_CODE;
                let start = i;
                let end = skip_ident_cont_avx2(content, i + 1);
                i = end;
                let len = end - start;
                if (2..=KW_MAX_LEN).contains(&len) {
                    let first = content[start];
                    let mid = content[start + len / 2];
                    let last = content[end - 1];
                    let h = kw_hash(first, mid, last, len);
                    let slot = KW_SLOT[h];
                    if slot != KW_EMPTY {
                        let idx = slot as usize;
                        if len == KEYWORD_LENS[idx] as usize
                            && &content[start..end] == KEYWORD_NAMES[idx].as_bytes()
                        {
                            acc.keyword_counts[idx] += 1;
                            acc.total_keywords += 1;
                        }
                    }
                }
            }
            TOK_SLASH => {
                if i + 1 < content_len && content[i + 1] == b'/' {
                    line_state |= STATE_COMMENT;
                    let Some(pos) = find_byte_avx2(&content[i + 2..], b'\n') else {
                        if (line_state & STATE_CODE) != 0 {
                            acc.code_lines += 1;
                        } else if (line_state & STATE_COMMENT) != 0 {
                            acc.comment_lines += 1;
                        } else {
                            acc.blank_lines += 1;
                        }
                        acc.lines_processed += 1;
                        break;
                    };
                    let nl_idx = i + 2 + pos;
                    if (line_state & STATE_CODE) != 0 {
                        acc.code_lines += 1;
                    } else if (line_state & STATE_COMMENT) != 0 {
                        acc.comment_lines += 1;
                    } else {
                        acc.blank_lines += 1;
                    }
                    acc.lines_processed += 1;
                    line_state = STATE_NONE;
                    i = nl_idx + 1;
                    continue;
                }
                line_state |= STATE_CODE;
                i += 1;
            }
            TOK_BACKSLASH => {
                if i + 1 < content_len && content[i + 1] == b'\\' {
                    line_state |= STATE_CODE;
                    let end_i = match find_byte_avx2(&content[i + 2..], b'\n') {
                        Some(pos) => i + 2 + pos,
                        None => content_len,
                    };
                    i = end_i;
                    continue;
                }
                line_state |= STATE_CODE;
                i += 1;
            }
            TOK_QUOTE => {
                line_state |= STATE_CODE;
                let end_i = skip_literal(content, i + 1, b'"');
                let nl = count_newlines_avx2(&content[i + 1..end_i]);
                if nl > 0 {
                    acc.code_lines += nl as i64;
                    acc.lines_processed += nl as i64;
                }
                i = end_i;
            }
            TOK_APOSTROPHE => {
                line_state |= STATE_CODE;
                let end_i = skip_literal(content, i + 1, b'\'');
                let nl = count_newlines_avx2(&content[i + 1..end_i]);
                if nl > 0 {
                    acc.code_lines += nl as i64;
                    acc.lines_processed += nl as i64;
                }
                i = end_i;
            }
            _ => {
                line_state |= STATE_CODE;
                i += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn find_kw_id(name: &str) -> usize {
        KEYWORD_NAMES.iter().position(|&kw| kw == name).unwrap()
    }

    #[test]
    fn test_zig_keyword_table_and_mph() {
        assert_eq!(KEYWORD_COUNT, 46);
        let mut seen = [false; 256];
        for i in 0..KEYWORD_COUNT {
            let kw = KEYWORD_NAMES[i];
            let len = kw.len();
            assert_eq!(len, KEYWORD_LENS[i] as usize);
            assert!((2..=KW_MAX_LEN).contains(&len));

            let bytes = kw.as_bytes();
            let first = bytes[0];
            let mid = bytes[len / 2];
            let last = bytes[len - 1];
            let h = kw_hash(first, mid, last, len);
            assert!(!seen[h]);
            seen[h] = true;
            assert_eq!(KW_SLOT[h], i as u8);

            let mut res = ParseResult::default();
            parse_into(bytes, &mut res);
            assert_eq!(res.total_keywords, 1);
            assert_eq!(res.keyword_counts[i], 1);
        }

        let non_keywords = ["std", "debug", "print", "ArrayList", "u32", "i64", "fns", "ifs"];
        for &nk in &non_keywords {
            let mut res = ParseResult::default();
            parse_into(nk.as_bytes(), &mut res);
            assert_eq!(res.total_keywords, 0);
        }
    }

    #[test]
    fn test_zig_extensions() {
        assert!(is_zig_extension(b"main.zig"));
        assert!(is_zig_extension(b"build.zig.zon"));
        assert!(is_zig_extension(b"config.zon"));
        assert!(is_zig_extension(b"UPPER.ZIG"));
        assert!(!is_zig_extension(b"zig"));
        assert!(!is_zig_extension(b".zig"));
        assert!(!is_zig_extension(b"main.c"));
    }

    #[test]
    fn test_zig_comments_and_strings() {
        let src = b"const std = @import(\"std\");\n\n// line comment\n/// doc comment\npub fn main() void {\n    const s = \"hello // not comment\";\n    const c = 'z';\n    return;\n}\n";
        let mut res = ParseResult::default();
        parse_into(src, &mut res);

        assert_eq!(res.keyword_counts[find_kw_id("const")], 3);
        assert_eq!(res.keyword_counts[find_kw_id("pub")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("fn")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("return")], 1);
        assert_eq!(res.total_keywords, 6);
        assert_eq!(res.lines_processed, 9);
        assert_eq!(res.code_lines, 6);
        assert_eq!(res.comment_lines, 2);
        assert_eq!(res.blank_lines, 1);
    }

    #[test]
    fn test_zig_multiline_strings() {
        let src = b"const s = \n    \\\\first line with fn and var\n    \\\\second line with pub and const\n;\n";
        let mut res = ParseResult::default();
        parse_into(src, &mut res);

        assert_eq!(res.keyword_counts[find_kw_id("const")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("fn")], 0);
        assert_eq!(res.keyword_counts[find_kw_id("pub")], 0);
        assert_eq!(res.total_keywords, 1);
        assert_eq!(res.lines_processed, 4);
        assert_eq!(res.code_lines, 4);
    }

    #[test]
    fn test_zig_avx2_boundaries_and_eof() {
        for len in [31, 32, 33, 63, 64, 65, 127, 128, 129] {
            let mut buf = vec![b' '; len];
            buf.extend_from_slice(b"pub fn main() void {\n    return;\n}\n");
            let mut res = ParseResult::default();
            parse_into(&buf, &mut res);
            assert_eq!(res.keyword_counts[find_kw_id("pub")], 1);
            assert_eq!(res.keyword_counts[find_kw_id("fn")], 1);
            assert_eq!(res.keyword_counts[find_kw_id("return")], 1);
            assert_eq!(res.total_keywords, 3);
        }

        let mut res = ParseResult::default();
        parse_into(b"// unclosed line comment at eof", &mut res);
        assert_eq!(res.total_keywords, 0);
        assert_eq!(res.comment_lines, 1);
        assert_eq!(res.lines_processed, 1);
    }
}
