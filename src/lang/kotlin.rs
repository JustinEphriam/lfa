use super::{
    compute_keyword_lens, compute_kw_max_len, compute_kw_min_len, handle_nested_c_slash_comments,
    handle_quoted_literal, handle_triple_or_quoted_literal, kw_hash_fml_add, make_kw_table_fml_add,
    make_token_kind, make_token_lut_lo, match_keyword_with, unified_parse_into, LangLexer,
    ParseResult, STATE_CODE,
};

pub const KEYWORD_COUNT: usize = 74;
const KW_SLOT_MASK: usize = 255;

pub const KEYWORD_NAMES: [&str; KEYWORD_COUNT] = [
    "actual",
    "abstract",
    "annotation",
    "as",
    "break",
    "by",
    "catch",
    "class",
    "companion",
    "const",
    "constructor",
    "continue",
    "crossinline",
    "data",
    "delegate",
    "do",
    "dynamic",
    "else",
    "enum",
    "expect",
    "external",
    "false",
    "field",
    "file",
    "final",
    "finally",
    "for",
    "fun",
    "get",
    "if",
    "import",
    "in",
    "infix",
    "init",
    "inline",
    "inner",
    "internal",
    "is",
    "lateinit",
    "noinline",
    "null",
    "object",
    "open",
    "operator",
    "out",
    "override",
    "package",
    "param",
    "private",
    "property",
    "protected",
    "public",
    "receiver",
    "reified",
    "return",
    "sealed",
    "set",
    "setparam",
    "super",
    "suspend",
    "tailrec",
    "this",
    "throw",
    "true",
    "try",
    "typealias",
    "typeof",
    "val",
    "value",
    "var",
    "vararg",
    "when",
    "where",
    "while",
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
    kw_hash_fml_add(first, mid, last, len, [1, 3, 389, 509], 1, KW_SLOT_MASK)
}

const KW_TABLE: [u8; 256] = make_kw_table_fml_add(&KEYWORD_NAMES, [1, 3, 389, 509], 1);

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
pub fn is_kotlin_extension(name: &[u8]) -> bool {
    super::detect_language(name, super::LANG_ALL_MASK) == Some(super::LangId::Kotlin)
}

pub struct KotlinLexer;

impl LangLexer<KEYWORD_COUNT> for KotlinLexer {
    const TOKEN_KIND: [u8; 256] = TOKEN_KIND;
    const TOKEN_LUT_LO: [i8; 32] = TOKEN_LUT_LO;
    const HAS_DOLLAR_IDENT: bool = false;
    const SUPPORTS_TEMPLATES: bool = false;
    const HAS_SPECIAL_IDENT: bool = false;
    const HAS_LINE_START_HOOK: bool = false;

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
            TOK_SLASH => handle_nested_c_slash_comments(content, i, line_state, acc),
            TOK_QUOTE => handle_triple_or_quoted_literal(content, i, b'"', line_state, acc),
            TOK_APOSTROPHE => handle_quoted_literal(content, i, b'\'', line_state, acc),
            _ => {
                *line_state |= STATE_CODE;
                i + 1
            }
        }
    }
}

pub fn parse_into(content: &[u8], acc: &mut ParseResult<KEYWORD_COUNT>) {
    unified_parse_into::<KEYWORD_COUNT, KotlinLexer>(content, acc);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kotlin_extensions() {
        assert!(is_kotlin_extension(b"main.kt"));
        assert!(is_kotlin_extension(b"build.gradle.kts"));
        assert!(!is_kotlin_extension(b"main.k"));
    }

    #[test]
    fn test_kotlin_keyword_table_and_mph() {
        assert_eq!(KEYWORD_NAMES.len(), KEYWORD_COUNT);
        assert_eq!(KEYWORD_LENS.len(), KEYWORD_COUNT);
        for (i, name) in KEYWORD_NAMES.iter().enumerate() {
            let b = name.as_bytes();
            let h = kw_hash(b[0], b[b.len() / 2], b[b.len() - 1], b.len());
            assert_eq!(KW_TABLE[h], i as u8);
        }
    }

    #[test]
    fn test_kotlin_comments_and_strings() {
        let code = b"// single comment\n/* nested /* block */ comment */\npackage com.example\nclass Foo {\n    val s = \"\"\"\n    multiline\n    string\n    \"\"\"\n    fun bar() {}\n}\n";
        let mut acc = ParseResult::<KEYWORD_COUNT>::default();
        parse_into(code, &mut acc);
        assert_eq!(acc.comment_lines, 2);
        assert_eq!(acc.code_lines, 8);
        assert_eq!(acc.lines_processed, 10);
        assert_eq!(acc.total_keywords, 4);
        let eof_triple = b"val x = \"\"\"hello\"\"\"\n\n";
        let mut acc2 = ParseResult::<KEYWORD_COUNT>::default();
        parse_into(eof_triple, &mut acc2);
        assert_eq!(acc2.code_lines, 1);
        assert_eq!(acc2.blank_lines, 1);
        assert_eq!(acc2.lines_processed, 2);
    }
}
