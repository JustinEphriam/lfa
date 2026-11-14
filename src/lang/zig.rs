use super::{
    compute_keyword_lens, compute_kw_max_len, compute_kw_min_len, find_byte_avx2,
    finish_line_comment, handle_quoted_literal, kw_match_2_to_16, make_token_lut_lo,
    unified_parse_into, LangLexer, ParseResult, STATE_CODE, TOK_CODE, TOK_NL, TOK_WS,
};

pub const KEYWORD_COUNT: usize = 46;
const KW_SLOT_MASK: usize = 255;
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

pub const KEYWORD_LENS: [u8; KEYWORD_COUNT] = compute_keyword_lens(&KEYWORD_NAMES);
pub const KW_MIN_LEN: usize = compute_kw_min_len(&KEYWORD_NAMES);
pub const KW_MAX_LEN: usize = compute_kw_max_len(&KEYWORD_NAMES);

const _: () = assert!(KEYWORD_NAMES.len() == KEYWORD_COUNT);
const _: () = assert!(KEYWORD_LENS.len() == KEYWORD_COUNT);


#[inline(always)]
const fn kw_hash(first: u8, mid: u8, last: u8, len: usize) -> usize {
    let val = (first as usize)
        .wrapping_mul(1204)
        .wrapping_add((mid as usize).wrapping_mul(3183))
        .wrapping_add((last as usize).wrapping_mul(1459))
        .wrapping_add(len.wrapping_mul(3884));
    (val ^ (val >> 1)) & KW_SLOT_MASK
}

const KW_TABLE: [u8; 256] = {
    let mut table = [KW_EMPTY; 256];
    let mut i = 0;
    while i < KEYWORD_COUNT {
        let name = KEYWORD_NAMES[i].as_bytes();
        let len = KEYWORD_LENS[i] as usize;
        assert!(name.len() == len);
        assert!(len >= KW_MIN_LEN && len <= KW_MAX_LEN);
        let h = kw_hash(name[0], name[len / 2], name[len - 1], len);
        assert!(table[h] == KW_EMPTY);
        table[h] = i as u8;
        i += 1;
    }
    table
};

const TOK_IDENT: u8 = 3;
const TOK_SLASH: u8 = 4;
const TOK_QUOTE: u8 = 5;
const TOK_APOSTROPHE: u8 = 6;
const TOK_BACKSLASH: u8 = 7;

pub const TOKEN_KIND: [u8; 256] = {
    let mut t = [TOK_CODE; 256];
    t[b' ' as usize] = TOK_WS;
    t[b'\t' as usize] = TOK_WS;
    t[b'\r' as usize] = TOK_WS;
    t[0x0b_usize] = TOK_WS;
    t[0x0c_usize] = TOK_WS;
    t[b'\n' as usize] = TOK_NL;

    let mut c = b'a';
    while c <= b'z' {
        t[c as usize] = TOK_IDENT;
        c += 1;
    }
    let mut c = b'A';
    while c <= b'Z' {
        t[c as usize] = TOK_IDENT;
        c += 1;
    }
    t[b'_' as usize] = TOK_IDENT;

    t[b'/' as usize] = TOK_SLASH;
    t[b'"' as usize] = TOK_QUOTE;
    t[b'\'' as usize] = TOK_APOSTROPHE;
    t[b'\\' as usize] = TOK_BACKSLASH;
    t
};

pub const TOKEN_LUT_LO: [i8; 32] = make_token_lut_lo(&TOKEN_KIND);

#[cfg(test)]
#[inline(always)]
pub fn is_zig_extension(name: &[u8]) -> bool {
    super::detect_language(name, super::LANG_ALL_MASK) == Some(super::LangId::Zig)
}

pub struct ZigLexer;

impl LangLexer<KEYWORD_COUNT> for ZigLexer {
    const TOKEN_KIND: [u8; 256] = TOKEN_KIND;
    const TOKEN_LUT_LO: [i8; 32] = TOKEN_LUT_LO;
    const HAS_DOLLAR_IDENT: bool = false;
    const SUPPORTS_TEMPLATES: bool = false;

    #[inline(always)]
    fn match_keyword(cand: &[u8]) -> Option<usize> {
        let n = cand.len();
        if n.wrapping_sub(KW_MIN_LEN) <= KW_MAX_LEN - KW_MIN_LEN {
            let h = kw_hash(cand[0], cand[n / 2], cand[n - 1], n);
            let slot = KW_TABLE[h];
            if slot != KW_EMPTY {
                let idx = slot as usize;
                if len_matches(n, idx)
                    && kw_match_2_to_16(cand, KEYWORD_NAMES[idx].as_bytes())
                {
                    return Some(idx);
                }
            }
        }
        None
    }

    #[inline(always)]
    fn handle_special(
        kind: u8,
        content: &[u8],
        i: usize,
        line_state: &mut u8,
        acc: &mut ParseResult<KEYWORD_COUNT>,
    ) -> usize {
        let content_len = content.len();
        match kind {
            TOK_SLASH => {
                if i + 1 < content_len && content[i + 1] == b'/' {
                    return finish_line_comment(content, i + 2, line_state, acc);
                }
                *line_state |= STATE_CODE;
                i + 1
            }
            TOK_BACKSLASH => {
                if i + 1 < content_len && content[i + 1] == b'\\' {
                    *line_state |= STATE_CODE;
                    let end_i = match find_byte_avx2(&content[i + 2..], b'\n') {
                        Some(pos) => i + 2 + pos,
                        None => content_len,
                    };
                    return end_i;
                }
                *line_state |= STATE_CODE;
                i + 1
            }
            TOK_QUOTE => handle_quoted_literal(content, i, b'"', line_state, acc),
            TOK_APOSTROPHE => handle_quoted_literal(content, i, b'\'', line_state, acc),
            _ => {
                *line_state |= STATE_CODE;
                i + 1
            }
        }
    }
}

#[inline(always)]
fn len_matches(len: usize, idx: usize) -> bool {
    len == KEYWORD_LENS[idx] as usize
}

#[inline(always)]
pub fn parse_into(content: &[u8], acc: &mut ParseResult<KEYWORD_COUNT>) {
    unified_parse_into::<KEYWORD_COUNT, ZigLexer>(content, acc);
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
            assert_eq!(KW_TABLE[h], i as u8);

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

        let mut wide = b"const s = ".to_vec();
        wide.extend(std::iter::repeat_n(b' ', 40));
        wide.extend_from_slice(b"\\\\fn pub const\n;\n");
        let mut res2 = ParseResult::default();
        parse_into(&wide, &mut res2);
        assert_eq!(res2.total_keywords, 1);
        assert_eq!(res2.keyword_counts[find_kw_id("const")], 1);
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
