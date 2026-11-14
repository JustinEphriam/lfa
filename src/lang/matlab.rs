use super::{
    compute_keyword_lens, compute_kw_max_len, compute_kw_min_len, finish_block_comment,
    finish_line_comment, handle_quoted_literal, kw_hash_fml_add, make_kw_table_fml_add,
    make_token_kind, make_token_lut_lo, match_keyword_with, skip_nested_2byte_comment,
    unified_parse_into, LangLexer, ParseResult, STATE_CODE,
};

pub const KEYWORD_COUNT: usize = 20;
const KW_SLOT_MASK: usize = 63;

pub const KEYWORD_NAMES: [&str; KEYWORD_COUNT] = [
    "break",
    "case",
    "catch",
    "classdef",
    "continue",
    "else",
    "elseif",
    "end",
    "for",
    "function",
    "global",
    "if",
    "otherwise",
    "parfor",
    "persistent",
    "return",
    "spmd",
    "switch",
    "try",
    "while",
];

pub const KEYWORD_LENS: [u8; KEYWORD_COUNT] = compute_keyword_lens(&KEYWORD_NAMES);
pub const KW_MIN_LEN: usize = compute_kw_min_len(&KEYWORD_NAMES);
pub const KW_MAX_LEN: usize = compute_kw_max_len(&KEYWORD_NAMES);

#[inline(always)]
const fn kw_hash(first: u8, mid: u8, last: u8, len: usize) -> usize {
    kw_hash_fml_add(first, mid, last, len, [1, 1, 1, 73], 1, KW_SLOT_MASK)
}

const KW_TABLE: [u8; 64] = make_kw_table_fml_add(&KEYWORD_NAMES, [1, 1, 1, 73], 1);

const TOK_IDENT: u8 = 3;
const TOK_PERCENT: u8 = 4;
const TOK_QUOTE: u8 = 5;
const TOK_APOSTROPHE: u8 = 6;

pub const TOKEN_KIND: [u8; 256] = make_token_kind(
    TOK_IDENT,
    &[
        (b'%', TOK_PERCENT),
        (b'"', TOK_QUOTE),
        (b'\'', TOK_APOSTROPHE),
    ],
);

pub const TOKEN_LUT_LO: [i8; 32] = make_token_lut_lo(&TOKEN_KIND);

#[cfg(test)]
#[inline(always)]
pub fn is_matlab_extension(name: &[u8]) -> bool {
    super::detect_language(name, super::LANG_ALL_MASK) == Some(super::LangId::Matlab)
}

pub struct MatlabLexer;

impl LangLexer<KEYWORD_COUNT> for MatlabLexer {
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
        let content_len = content.len();
        match kind {
            TOK_PERCENT => {
                if i + 1 < content_len && content[i + 1] == b'{' {
                    let end_i = skip_nested_2byte_comment(content, i + 2, b'%', b'{', b'%', b'}');
                    finish_block_comment(content, i + 2, end_i, line_state, acc)
                } else {
                    finish_line_comment(content, i + 1, line_state, acc)
                }
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

pub fn parse_into(content: &[u8], acc: &mut ParseResult<KEYWORD_COUNT>) {
    unified_parse_into::<KEYWORD_COUNT, MatlabLexer>(content, acc);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matlab_extensions() {
        assert!(is_matlab_extension(b"script.m"));
        assert!(!is_matlab_extension(b"main.c"));
        assert!(!is_matlab_extension(b"main.rs"));
    }

    #[test]
    fn test_matlab_keyword_table_and_mph() {
        for (i, name) in KEYWORD_NAMES.iter().enumerate() {
            let b = name.as_bytes();
            let h = kw_hash(b[0], b[b.len() / 2], b[b.len() - 1], b.len());
            assert_eq!(KW_TABLE[h], i as u8);
            assert_eq!(MatlabLexer::match_keyword(b), Some(i));
        }
        assert_eq!(MatlabLexer::match_keyword(b"notakeyword"), None);
        assert_eq!(MatlabLexer::match_keyword(b"matrix"), None);
    }

    #[test]
    fn test_matlab_comments_and_strings() {
        let src = b"% single line\nfunction y = square(x)\n%{\n  multi-line block\n%}\n    if x > 0\n        y = x * x;\n    else\n        y = 0;\n    end\nend\n";
        let mut acc = ParseResult::default();
        parse_into(src, &mut acc);
        assert_eq!(acc.comment_lines, 4);
        assert_eq!(acc.code_lines, 7);
        assert_eq!(acc.blank_lines, 0);
        assert!(acc.total_keywords >= 4);
    }
}
