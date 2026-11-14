use super::{
    compute_keyword_lens, compute_kw_max_len, compute_kw_min_len, finish_line_comment,
    handle_nested_2byte_comment, handle_quoted_literal, kw_hash_fml_add, make_kw_table_fml_add,
    make_token_kind, make_token_lut_lo, match_keyword_with, unified_parse_into, LangLexer,
    ParseResult, STATE_CODE,
};

pub const KEYWORD_COUNT: usize = 27;
const KW_SLOT_MASK: usize = 63;

pub const KEYWORD_NAMES: [&str; KEYWORD_COUNT] = [
    "block",
    "br",
    "call",
    "data",
    "drop",
    "elem",
    "else",
    "export",
    "func",
    "global",
    "if",
    "import",
    "local",
    "loop",
    "memory",
    "module",
    "mut",
    "nop",
    "offset",
    "param",
    "result",
    "return",
    "select",
    "table",
    "then",
    "type",
    "unreachable",
];

pub const KEYWORD_LENS: [u8; KEYWORD_COUNT] = compute_keyword_lens(&KEYWORD_NAMES);
pub const KW_MIN_LEN: usize = compute_kw_min_len(&KEYWORD_NAMES);
pub const KW_MAX_LEN: usize = compute_kw_max_len(&KEYWORD_NAMES);

#[inline(always)]
const fn kw_hash(first: u8, mid: u8, last: u8, len: usize) -> usize {
    kw_hash_fml_add(first, mid, last, len, [660, 9553, 6173, 2466], 1, KW_SLOT_MASK)
}

const KW_TABLE: [u8; 64] = make_kw_table_fml_add(&KEYWORD_NAMES, [660, 9553, 6173, 2466], 1);

const TOK_IDENT: u8 = 3;
const TOK_SEMICOLON: u8 = 4;
const TOK_LPAREN: u8 = 5;
const TOK_QUOTE: u8 = 6;

pub const TOKEN_KIND: [u8; 256] = make_token_kind(
    TOK_IDENT,
    &[
        (b'-', TOK_IDENT),
        (b'.', TOK_IDENT),
        (b';', TOK_SEMICOLON),
        (b'(', TOK_LPAREN),
        (b'"', TOK_QUOTE),
    ],
);

pub const TOKEN_LUT_LO: [i8; 32] = make_token_lut_lo(&TOKEN_KIND);

pub struct WatLexer;

impl LangLexer<KEYWORD_COUNT> for WatLexer {
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
        let content_len = content.len();
        match kind {
            TOK_SEMICOLON => {
                if i + 1 < content_len && content[i + 1] == b';' {
                    finish_line_comment(content, i + 2, line_state, acc)
                } else {
                    *line_state |= STATE_CODE;
                    i + 1
                }
            }
            TOK_LPAREN => {
                handle_nested_2byte_comment(content, i, *b"(;", *b";)", line_state, acc)
            }
            TOK_QUOTE => handle_quoted_literal(content, i, b'"', line_state, acc),
            _ => {
                *line_state |= STATE_CODE;
                i + 1
            }
        }
    }
}

pub fn parse_into(content: &[u8], res: &mut ParseResult<KEYWORD_COUNT>) {
    unified_parse_into::<KEYWORD_COUNT, WatLexer>(content, res);
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_wat_keyword_table_and_mph() {
        for (i, &name) in KEYWORD_NAMES.iter().enumerate() {
            let idx = WatLexer::match_keyword(name.as_bytes());
            assert_eq!(idx, Some(i), "keyword {} failed match", name);
        }
        assert_eq!(WatLexer::match_keyword(b"notakeyword"), None);
    }

    #[test]
    fn test_wat_comments_and_strings() {
        let code = b";; WebAssembly text module\n(module\n  (func $add (param $lhs i32) (param $rhs i32) (result i32)\n    (; nested (; block ;) comment ;)\n    local.get $lhs\n    local.get $rhs\n    i32.add)\n  (export \"add\" (func $add))\n)\n";
        let mut res = ParseResult::default();
        parse_into(code, &mut res);
        assert_eq!(res.lines_processed, 9);
        assert_eq!(res.comment_lines, 2);
        assert_eq!(res.code_lines, 7);
        assert_eq!(res.blank_lines, 0);
        assert!(res.total_keywords >= 5);
    }
}
