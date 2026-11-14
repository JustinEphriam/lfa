use super::{
    compute_keyword_lens, compute_kw_max_len, compute_kw_min_len, handle_c_slash_comments,
    handle_quoted_literal, handle_triple_or_quoted_literal, kw_hash_fml_add, make_kw_table_fml_add,
    make_token_kind, make_token_lut_lo, match_keyword_with, unified_parse_into, LangLexer,
    ParseResult, STATE_CODE,
};

pub const KEYWORD_COUNT: usize = 55;
const KW_SLOT_MASK: usize = 255;

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

const TOK_IDENT: u8 = 3;
const TOK_SLASH: u8 = 4;
const TOK_QUOTE: u8 = 5;
const TOK_APOSTROPHE: u8 = 6;

#[inline(always)]
const fn kw_hash(first: u8, mid: u8, last: u8, len: usize) -> usize {
    kw_hash_fml_add(first, mid, last, len, [1, 12, 2276, 17], 1, KW_SLOT_MASK)
}

const KW_TABLE: [u8; 256] = make_kw_table_fml_add(&KEYWORD_NAMES, [1, 12, 2276, 17], 1);

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
        match_keyword_with(
            cand,
            KW_MIN_LEN,
            KW_MAX_LEN,
            &KW_TABLE,
            &KEYWORD_LENS,
            &KEYWORD_NAMES,
            |c| kw_hash(c[0], c[c.len() / 2], c[c.len() - 1], c.len()),
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
