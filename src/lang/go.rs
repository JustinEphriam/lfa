use super::{
    compute_keyword_lens, compute_kw_max_len, compute_kw_min_len, count_newlines_avx2,
    find_byte_avx2, handle_c_slash_comments, handle_quoted_literal, kw_hash_fsl_add,
    make_kw_table_fsl_add, make_token_kind, make_token_lut_lo, match_keyword_with,
    unified_parse_into, LangLexer, ParseResult, STATE_CODE,
};

pub const KEYWORD_COUNT: usize = 25;
const KW_SLOT_MASK: usize = 255;

pub const KEYWORD_NAMES: [&str; KEYWORD_COUNT] = [
    "break",
    "case",
    "chan",
    "const",
    "continue",
    "default",
    "defer",
    "else",
    "fallthrough",
    "for",
    "func",
    "go",
    "goto",
    "if",
    "import",
    "interface",
    "map",
    "package",
    "range",
    "return",
    "select",
    "struct",
    "switch",
    "type",
    "var",
];

pub const KEYWORD_LENS: [u8; KEYWORD_COUNT] = compute_keyword_lens(&KEYWORD_NAMES);
pub const KW_MIN_LEN: usize = compute_kw_min_len(&KEYWORD_NAMES);
pub const KW_MAX_LEN: usize = compute_kw_max_len(&KEYWORD_NAMES);

const TOK_IDENT: u8 = 3;
const TOK_SLASH: u8 = 4;
const TOK_QUOTE: u8 = 5;
const TOK_APOSTROPHE: u8 = 6;
const TOK_BACKTICK: u8 = 7;


#[inline(always)]
const fn kw_hash(first: u8, second: u8, last: u8, len: usize) -> usize {
    kw_hash_fsl_add(first, second, last, len, [2066, 654, 3810, 320], 1, KW_SLOT_MASK)
}

const KW_TABLE: [u8; 256] = make_kw_table_fsl_add(&KEYWORD_NAMES, [2066, 654, 3810, 320], 1);

pub const TOKEN_KIND: [u8; 256] = make_token_kind(
    TOK_IDENT,
    &[
        (b'/', TOK_SLASH),
        (b'"', TOK_QUOTE),
        (b'\'', TOK_APOSTROPHE),
        (b'`', TOK_BACKTICK),
    ],
);

pub const TOKEN_LUT_LO: [i8; 32] = make_token_lut_lo(&TOKEN_KIND);

#[cfg(test)]
#[inline(always)]
pub fn is_go_extension(name: &[u8]) -> bool {
    super::detect_language(name, super::LANG_ALL_MASK) == Some(super::LangId::Go)
}

pub struct GoLexer;

impl LangLexer<KEYWORD_COUNT> for GoLexer {
    const TOKEN_KIND: [u8; 256] = TOKEN_KIND;
    const TOKEN_LUT_LO: [i8; 32] = TOKEN_LUT_LO;
    const HAS_DOLLAR_IDENT: bool = false;
    const SUPPORTS_TEMPLATES: bool = false;

    #[inline(always)]
    fn match_keyword(cand: &[u8]) -> Option<usize> {
        match_keyword_with(
            cand,
            KW_MIN_LEN,
            KW_MAX_LEN,
            &KW_TABLE,
            &KEYWORD_LENS,
            &KEYWORD_NAMES,
            |c| kw_hash(c[0], c[1], c[c.len() - 1], c.len()),
        )
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
            TOK_SLASH => handle_c_slash_comments(content, i, line_state, acc),
            TOK_QUOTE => handle_quoted_literal(content, i, b'"', line_state, acc),
            TOK_APOSTROPHE => handle_quoted_literal(content, i, b'\'', line_state, acc),
            TOK_BACKTICK => {
                *line_state |= STATE_CODE;
                let end_i = match find_byte_avx2(&content[i + 1..], b'`') {
                    Some(pos) => i + 1 + pos + 1,
                    None => content_len,
                };
                let nl = count_newlines_avx2(&content[i + 1..end_i]);
                if nl > 0 {
                    acc.code_lines += nl as i64;
                    acc.lines_processed += nl as i64;
                }
                end_i
            }
            _ => {
                *line_state |= STATE_CODE;
                i + 1
            }
        }
    }
}

#[inline(always)]
pub fn parse_into(content: &[u8], acc: &mut ParseResult<KEYWORD_COUNT>) {
    unified_parse_into::<KEYWORD_COUNT, GoLexer>(content, acc);
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn find_kw_id(name: &str) -> usize {
        KEYWORD_NAMES.iter().position(|&kw| kw == name).unwrap()
    }

    #[test]
    fn test_go_keyword_table_and_mph() {
        assert_eq!(KEYWORD_COUNT, 25);
        let mut seen = [false; 256];
        for i in 0..KEYWORD_COUNT {
            let kw = KEYWORD_NAMES[i];
            let len = kw.len();
            assert_eq!(len, KEYWORD_LENS[i] as usize);
            assert!((2..=KW_MAX_LEN).contains(&len));

            let bytes = kw.as_bytes();
            let first = bytes[0];
            let second = bytes[1];
            let last = bytes[len - 1];
            let h = kw_hash(first, second, last, len);
            assert!(!seen[h]);
            seen[h] = true;
            assert_eq!(KW_TABLE[h], i as u8);

            let mut res = ParseResult::default();
            parse_into(bytes, &mut res);
            assert_eq!(res.total_keywords, 1);
            assert_eq!(res.keyword_counts[i], 1);
        }

        let non_keywords = [
            "main", "foo", "fmt", "Println", "string", "int", "true", "false", "nil", "make",
            "len", "break1", "gos", "ifs",
        ];
        for &nk in &non_keywords {
            let mut res = ParseResult::default();
            parse_into(nk.as_bytes(), &mut res);
            assert_eq!(res.total_keywords, 0);
        }
    }

    #[test]
    fn test_go_extensions() {
        assert!(is_go_extension(b"main.go"));
        assert!(is_go_extension(b"server.GO"));
        assert!(is_go_extension(b"path/to/file.go"));
        assert!(!is_go_extension(b"main.g"));
        assert!(!is_go_extension(b"main.c"));
        assert!(!is_go_extension(b"go"));
        assert!(!is_go_extension(b".go"));
    }

    #[test]
    fn test_go_comments_and_strings() {
        let src = b"package main\n\n// line comment\nfunc main() {\n    /* block\n       comment */\n    s := \"hello // not a comment\"\n    r := 'a'\n    return\n}\n";
        let mut res = ParseResult::default();
        parse_into(src, &mut res);

        assert_eq!(res.keyword_counts[find_kw_id("package")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("func")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("return")], 1);
        assert_eq!(res.total_keywords, 3);
        assert_eq!(res.lines_processed, 10);
        assert_eq!(res.code_lines, 6);
        assert_eq!(res.comment_lines, 3);
        assert_eq!(res.blank_lines, 1);
    }

    #[test]
    fn test_go_raw_strings() {
        let src = b"package main\n\nconst raw = `line 1\nline 2 with func and var\nline 3`\n\nvar x = 1\n";
        let mut res = ParseResult::default();
        parse_into(src, &mut res);

        assert_eq!(res.keyword_counts[find_kw_id("package")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("const")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("var")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("func")], 0);
        assert_eq!(res.total_keywords, 3);
        assert_eq!(res.code_lines, 5);
        assert_eq!(res.comment_lines, 0);
        assert_eq!(res.blank_lines, 2);
    }

    #[test]
    fn test_go_lines_code_comments_blanks() {
        let src = b"// Copyright\n\npackage main\n\nfunc main() { /* start\nend */\n\n}\n";
        let mut res = ParseResult::default();
        parse_into(src, &mut res);

        assert_eq!(res.lines_processed, 8);
        assert_eq!(res.code_lines, 3);
        assert_eq!(res.comment_lines, 2);
        assert_eq!(res.blank_lines, 3);
    }

    #[test]
    fn test_go_avx2_boundaries_and_eof() {
        for len in [31, 32, 33, 63, 64, 65, 127, 128, 129] {
            let mut buf = vec![b' '; len];
            buf.extend_from_slice(b"func foo() {\n    return\n}\n");
            let mut res = ParseResult::default();
            parse_into(&buf, &mut res);
            assert_eq!(res.keyword_counts[find_kw_id("func")], 1);
            assert_eq!(res.keyword_counts[find_kw_id("return")], 1);
            assert_eq!(res.total_keywords, 2);
        }

        let mut res = ParseResult::default();
        parse_into(b"/* unclosed comment", &mut res);
        assert_eq!(res.total_keywords, 0);

        let mut res2 = ParseResult::default();
        parse_into(b"`unclosed raw string", &mut res2);
        assert_eq!(res2.total_keywords, 0);

        let mut res3 = ParseResult::default();
        parse_into(b"// unclosed line comment at eof", &mut res3);
        assert_eq!(res3.total_keywords, 0);
        assert_eq!(res3.comment_lines, 1);
        assert_eq!(res3.lines_processed, 1);
    }
}
