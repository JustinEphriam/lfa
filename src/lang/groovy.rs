use super::{
    compute_keyword_lens, compute_kw_max_len, compute_kw_min_len, finish_line_comment,
    handle_spliced_c_slash_comments, handle_triple_or_quoted_literal, kw_hash_fml_add,
    make_kw_table_fml_add, make_token_kind, make_token_lut_lo, match_keyword_with,
    unified_parse_into, LangLexer, ParseResult, STATE_CODE,
};

pub const KEYWORD_COUNT: usize = 50;
const KW_SLOT_MASK: usize = 255;

pub const KEYWORD_NAMES: [&str; KEYWORD_COUNT] = [
    "as",
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
    "def",
    "default",
    "do",
    "double",
    "else",
    "enum",
    "extends",
    "false",
    "finally",
    "float",
    "for",
    "goto",
    "if",
    "implements",
    "import",
    "in",
    "instanceof",
    "int",
    "interface",
    "long",
    "native",
    "new",
    "null",
    "package",
    "return",
    "short",
    "super",
    "switch",
    "this",
    "throw",
    "throws",
    "trait",
    "true",
    "try",
    "var",
    "void",
    "volatile",
    "while",
    "yield",
];

pub const KEYWORD_LENS: [u8; KEYWORD_COUNT] = compute_keyword_lens(&KEYWORD_NAMES);
pub const KW_MIN_LEN: usize = compute_kw_min_len(&KEYWORD_NAMES);
pub const KW_MAX_LEN: usize = compute_kw_max_len(&KEYWORD_NAMES);

#[inline(always)]
const fn kw_hash(first: u8, mid: u8, last: u8, len: usize) -> usize {
    kw_hash_fml_add(first, mid, last, len, [6554, 6021, 1589, 8039], 1, KW_SLOT_MASK)
}

const KW_TABLE: [u8; 256] = make_kw_table_fml_add(&KEYWORD_NAMES, [6554, 6021, 1589, 8039], 1);

const TOK_IDENT: u8 = 3;
const TOK_SLASH: u8 = 4;
const TOK_QUOTE: u8 = 5;
const TOK_APOSTROPHE: u8 = 6;
const TOK_HASH: u8 = 7;

pub const TOKEN_KIND: [u8; 256] = make_token_kind(
    TOK_IDENT,
    &[
        (b'$', TOK_IDENT),
        (b'/', TOK_SLASH),
        (b'"', TOK_QUOTE),
        (b'\'', TOK_APOSTROPHE),
        (b'#', TOK_HASH),
    ],
);

pub const TOKEN_LUT_LO: [i8; 32] = make_token_lut_lo(&TOKEN_KIND);

pub struct GroovyLexer;

impl LangLexer<KEYWORD_COUNT> for GroovyLexer {
    const TOKEN_KIND: [u8; 256] = TOKEN_KIND;
    const TOKEN_LUT_LO: [i8; 32] = TOKEN_LUT_LO;
    const HAS_DOLLAR_IDENT: bool = true;
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
            TOK_HASH => finish_line_comment(content, i + 1, line_state, acc),
            TOK_SLASH => handle_spliced_c_slash_comments(content, i, line_state, acc),
            TOK_QUOTE => handle_triple_or_quoted_literal(content, i, b'"', line_state, acc),
            TOK_APOSTROPHE => handle_triple_or_quoted_literal(content, i, b'\'', line_state, acc),
            _ => {
                *line_state |= STATE_CODE;
                i + 1
            }
        }
    }
}

pub fn parse_into(content: &[u8], res: &mut ParseResult<KEYWORD_COUNT>) {
    unified_parse_into::<KEYWORD_COUNT, GroovyLexer>(content, res);
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_groovy_keyword_table_and_mph() {
        for (i, &name) in KEYWORD_NAMES.iter().enumerate() {
            let idx = GroovyLexer::match_keyword(name.as_bytes());
            assert_eq!(idx, Some(i), "keyword {} failed match", name);
        }
        assert_eq!(GroovyLexer::match_keyword(b"notakeyword"), None);
        assert_eq!(GroovyLexer::match_keyword(b"x"), None);
    }

    #[test]
    fn test_groovy_comments_and_strings() {
        let code = b"// Groovy script\n\
                     class Example {\n\
                         static void main(String[] args) {\n\
                             def msg = '''multiline\n\
                             string'''\n\
                             /* comment */\n\
                             println(msg)\n\
                         }\n\
                     }\n";
        let mut res = ParseResult::default();
        parse_into(code, &mut res);
        assert_eq!(res.lines_processed, 9);
        assert_eq!(res.comment_lines, 2);
        assert_eq!(res.code_lines, 7);
        assert_eq!(res.blank_lines, 0);
        assert_eq!(res.total_keywords, 3);
    }
}
