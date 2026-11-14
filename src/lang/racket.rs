use super::{
    compute_keyword_lens, compute_kw_max_len, compute_kw_min_len, finish_block_comment,
    finish_line_comment, handle_quoted_literal, kw_hash_fml_add, make_kw_table_fml_add,
    make_token_kind, make_token_lut_lo, match_keyword_with, skip_lisp_datum,
    skip_nested_2byte_comment, unified_parse_into, LangLexer, ParseResult, STATE_CODE,
};

pub const KEYWORD_COUNT: usize = 15;
const KW_SLOT_MASK: usize = 31;

pub const KEYWORD_NAMES: [&str; KEYWORD_COUNT] = [
    "and",
    "cond",
    "define",
    "else",
    "if",
    "lambda",
    "let",
    "match",
    "module",
    "or",
    "provide",
    "require",
    "struct",
    "unless",
    "when",
];

pub const KEYWORD_LENS: [u8; KEYWORD_COUNT] = compute_keyword_lens(&KEYWORD_NAMES);
pub const KW_MIN_LEN: usize = compute_kw_min_len(&KEYWORD_NAMES);
pub const KW_MAX_LEN: usize = compute_kw_max_len(&KEYWORD_NAMES);

#[inline(always)]
const fn kw_hash(len: usize, first: u8, mid: u8, last: u8) -> usize {
    kw_hash_fml_add(first, mid, last, len, [9604, 5944, 8434, 1395], 1, KW_SLOT_MASK)
}

const KW_TABLE: [u8; 32] = make_kw_table_fml_add(&KEYWORD_NAMES, [9604, 5944, 8434, 1395], 1);

const TOK_IDENT: u8 = 3;
const TOK_SEMICOLON: u8 = 4;
const TOK_HASH: u8 = 5;
const TOK_QUOTE: u8 = 6;

pub const TOKEN_KIND: [u8; 256] = make_token_kind(
    TOK_IDENT,
    &[
        (b'-', TOK_IDENT),
        (b'?', TOK_IDENT),
        (b'!', TOK_IDENT),
        (b';', TOK_SEMICOLON),
        (b'#', TOK_HASH),
        (b'"', TOK_QUOTE),
    ],
);

pub const TOKEN_LUT_LO: [i8; 32] = make_token_lut_lo(&TOKEN_KIND);

pub struct RacketLexer;

impl LangLexer<KEYWORD_COUNT> for RacketLexer {
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
            |c| kw_hash(c.len(), c[0], c[c.len() / 2], c[c.len() - 1]),
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
            TOK_SEMICOLON => finish_line_comment(content, i + 1, line_state, acc),
            TOK_HASH => {
                if i + 1 < content_len && content[i + 1] == b'|' {
                    let end_i = skip_nested_2byte_comment(content, i + 2, b'#', b'|', b'|', b'#');
                    finish_block_comment(content, i + 2, end_i, line_state, acc)
                } else if i + 1 < content_len && content[i + 1] == b';' {
                    let end_i = skip_lisp_datum(content, i + 2);
                    finish_block_comment(content, i + 2, end_i, line_state, acc)
                } else {
                    *line_state |= STATE_CODE;
                    i + 1
                }
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
    unified_parse_into::<KEYWORD_COUNT, RacketLexer>(content, res);
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_racket_keyword_table_and_mph() {
        for (i, &name) in KEYWORD_NAMES.iter().enumerate() {
            let idx = RacketLexer::match_keyword(name.as_bytes());
            assert_eq!(idx, Some(i), "keyword {} failed match", name);
        }
        assert_eq!(RacketLexer::match_keyword(b"notakeyword"), None);
    }
}
