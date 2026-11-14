use super::{
    compute_keyword_lens, compute_kw_max_len, compute_kw_min_len, handle_c_slash_comments,
    handle_quoted_literal, handle_triple_or_quoted_literal, kw_match_2_to_16, make_token_kind,
    make_token_lut_lo, unified_parse_into, LangLexer, ParseResult, STATE_CODE,
};

pub const KEYWORD_COUNT: usize = 55;
const KW_SLOT_MASK: usize = 255;
const KW_EMPTY: u8 = 255;

pub const KEYWORD_NAMES: [&str; KEYWORD_COUNT] = [
    "abstract",
    "assert",
    "boolean",
    "break",
    "byte",
    "case",
    "catch",
    "char",
    "class",
    "const",
    "continue",
    "default",
    "do",
    "double",
    "else",
    "enum",
    "extends",
    "false",
    "final",
    "finally",
    "float",
    "for",
    "goto",
    "if",
    "implements",
    "import",
    "instanceof",
    "int",
    "interface",
    "long",
    "native",
    "new",
    "null",
    "package",
    "private",
    "protected",
    "public",
    "record",
    "return",
    "short",
    "static",
    "strictfp",
    "super",
    "switch",
    "synchronized",
    "this",
    "throw",
    "throws",
    "transient",
    "true",
    "try",
    "void",
    "volatile",
    "while",
    "yield",
];

pub const KEYWORD_LENS: [u8; KEYWORD_COUNT] = compute_keyword_lens(&KEYWORD_NAMES);
pub const KW_MIN_LEN: usize = compute_kw_min_len(&KEYWORD_NAMES);
pub const KW_MAX_LEN: usize = compute_kw_max_len(&KEYWORD_NAMES);

const _: () = assert!(KEYWORD_NAMES.len() == KEYWORD_COUNT);
const _: () = assert!(KEYWORD_LENS.len() == KEYWORD_COUNT);

const TOK_IDENT: u8 = 3;
const TOK_SLASH: u8 = 4;
const TOK_QUOTE: u8 = 5;
const TOK_APOSTROPHE: u8 = 6;

#[inline(always)]
const fn kw_hash(first: u8, mid: u8, last: u8, len: usize) -> usize {
    let val = (first as usize)
        .wrapping_add((mid as usize).wrapping_mul(12))
        .wrapping_add((last as usize).wrapping_mul(2276))
        .wrapping_add(len.wrapping_mul(17));
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

pub const TOKEN_KIND: [u8; 256] = make_token_kind(
    TOK_IDENT,
    &[
        (b'/', TOK_SLASH),
        (b'"', TOK_QUOTE),
        (b'\'', TOK_APOSTROPHE),
    ],
);

pub const TOKEN_LUT_LO: [i8; 32] = make_token_lut_lo(&TOKEN_KIND);

#[cfg(test)]
#[inline(always)]
pub fn is_java_extension(name: &[u8]) -> bool {
    super::detect_language(name, super::LANG_ALL_MASK) == Some(super::LangId::Java)
}

pub struct JavaLexer;

impl LangLexer<KEYWORD_COUNT> for JavaLexer {
    const TOKEN_KIND: [u8; 256] = TOKEN_KIND;
    const TOKEN_LUT_LO: [i8; 32] = TOKEN_LUT_LO;
    const HAS_DOLLAR_IDENT: bool = false;
    const SUPPORTS_TEMPLATES: bool = false;

    #[inline(always)]
    fn match_keyword(cand: &[u8]) -> Option<usize> {
        let n = cand.len();
        if n.wrapping_sub(KW_MIN_LEN) <= KW_MAX_LEN - KW_MIN_LEN {
            let h = kw_hash(cand[0], cand[n / 2], cand[n - 1], n);
            let id = KW_TABLE[h];
            if id != KW_EMPTY {
                let idx = id as usize;
                debug_assert!(idx < KEYWORD_COUNT);
                if KEYWORD_LENS[idx] as usize == n
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
        match kind {
            TOK_SLASH => handle_c_slash_comments(content, i, line_state, acc),
            TOK_QUOTE => handle_triple_or_quoted_literal(content, i, b'"', line_state, acc),
            TOK_APOSTROPHE => handle_quoted_literal(content, i, b'\'', line_state, acc),
            _ => {
                *line_state |= STATE_CODE;
                i + 1
            }
        }
    }
}

#[inline(always)]
pub fn parse_into(content: &[u8], acc: &mut ParseResult<KEYWORD_COUNT>) {
    unified_parse_into::<KEYWORD_COUNT, JavaLexer>(content, acc);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_java_keyword_table_and_mph() {
        for (i, &name) in KEYWORD_NAMES.iter().enumerate() {
            let b = name.as_bytes();
            let idx = JavaLexer::match_keyword(b);
            assert_eq!(idx, Some(i), "keyword {}", name);
        }

        assert_eq!(JavaLexer::match_keyword(b"notakeyword"), None);
        assert_eq!(JavaLexer::match_keyword(b"x"), None);
        assert_eq!(JavaLexer::match_keyword(b"abstractx"), None);
        assert_eq!(JavaLexer::match_keyword(b""), None);
    }

    #[test]
    fn test_java_extensions() {
        assert!(is_java_extension(b"Main.java"));
        assert!(is_java_extension(b"path/to/Foo.JAVA"));
        assert!(!is_java_extension(b"Main.jav"));
        assert!(!is_java_extension(b"Main.class"));
        assert!(!is_java_extension(b"Main.jar"));
    }

    #[test]
    fn test_java_lines_code_comments_blanks() {
        let src = b"package com.example;\n\n// line comment\npublic class Foo {\n    /* block\n       comment */\n    final String s = \"hello\\nworld\";\n}\n";
        let mut res = ParseResult::<KEYWORD_COUNT>::default();
        parse_into(src, &mut res);

        assert_eq!(res.lines_processed, 8);
        assert_eq!(res.blank_lines, 1);
        assert_eq!(res.comment_lines, 3);
        assert_eq!(res.code_lines, 4);
        assert_eq!(res.total_keywords, 4);
    }

    #[test]
    fn test_java_text_blocks() {
        let src = b"class Test {\n    String block = \"\"\"\n        line 1\n        line 2\n        \"\"\";\n}\n";
        let mut res = ParseResult::<KEYWORD_COUNT>::default();
        parse_into(src, &mut res);

        assert_eq!(res.lines_processed, 6);
        assert_eq!(res.code_lines, 6);
        assert_eq!(res.comment_lines, 0);
        assert_eq!(res.blank_lines, 0);
    }
}
