use super::{
    compute_keyword_lens, compute_kw_max_len, compute_kw_min_len, handle_quoted_literal,
    handle_spliced_c_slash_comments, kw_hash_fml_add, make_kw_table_fml_add, make_token_kind,
    make_token_lut_lo, match_keyword_with, unified_parse_into, LangLexer, ParseResult, STATE_CODE,
};

pub const KEYWORD_COUNT: usize = 61;
const KW_SLOT_MASK: usize = 255;

pub const KEYWORD_NAMES: [&str; KEYWORD_COUNT] = [
    "abstract",
    "address",
    "anonymous",
    "as",
    "assembly",
    "bool",
    "break",
    "bytes",
    "calldata",
    "catch",
    "constant",
    "constructor",
    "continue",
    "contract",
    "delete",
    "do",
    "else",
    "emit",
    "enum",
    "event",
    "external",
    "fallback",
    "false",
    "for",
    "function",
    "if",
    "immutable",
    "import",
    "indexed",
    "interface",
    "internal",
    "is",
    "library",
    "mapping",
    "memory",
    "modifier",
    "new",
    "override",
    "payable",
    "pragma",
    "private",
    "public",
    "pure",
    "receive",
    "return",
    "returns",
    "revert",
    "storage",
    "string",
    "struct",
    "super",
    "this",
    "throw",
    "true",
    "try",
    "type",
    "unchecked",
    "using",
    "view",
    "virtual",
    "while",
];

pub const KEYWORD_LENS: [u8; KEYWORD_COUNT] = compute_keyword_lens(&KEYWORD_NAMES);
pub const KW_MIN_LEN: usize = compute_kw_min_len(&KEYWORD_NAMES);
pub const KW_MAX_LEN: usize = compute_kw_max_len(&KEYWORD_NAMES);

#[inline(always)]
const fn kw_hash(first: u8, mid: u8, last: u8, len: usize) -> usize {
    kw_hash_fml_add(first, mid, last, len, [1641, 5027, 8864, 7547], 4, KW_SLOT_MASK)
}

const KW_TABLE: [u8; 256] = make_kw_table_fml_add(&KEYWORD_NAMES, [1641, 5027, 8864, 7547], 4);

const TOK_IDENT: u8 = 3;
const TOK_SLASH: u8 = 4;
const TOK_QUOTE: u8 = 5;
const TOK_APOSTROPHE: u8 = 6;

pub const TOKEN_KIND: [u8; 256] = make_token_kind(
    TOK_IDENT,
    &[
        (b'$', TOK_IDENT),
        (b'/', TOK_SLASH),
        (b'"', TOK_QUOTE),
        (b'\'', TOK_APOSTROPHE),
    ],
);

pub const TOKEN_LUT_LO: [i8; 32] = make_token_lut_lo(&TOKEN_KIND);

pub struct SolidityLexer;

impl LangLexer<KEYWORD_COUNT> for SolidityLexer {
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
            TOK_SLASH => handle_spliced_c_slash_comments(content, i, line_state, acc),
            TOK_QUOTE => handle_quoted_literal(content, i, b'"', line_state, acc),
            TOK_APOSTROPHE => handle_quoted_literal(content, i, b'\'', line_state, acc),
            _ => {
                *line_state |= STATE_CODE;
                i + 1
            }
        }
    }
}

pub fn parse_into(content: &[u8], res: &mut ParseResult<KEYWORD_COUNT>) {
    unified_parse_into::<KEYWORD_COUNT, SolidityLexer>(content, res);
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_solidity_keyword_table_and_mph() {
        for (i, &name) in KEYWORD_NAMES.iter().enumerate() {
            let idx = SolidityLexer::match_keyword(name.as_bytes());
            assert_eq!(idx, Some(i), "keyword {} failed match", name);
        }
        assert_eq!(SolidityLexer::match_keyword(b"notakeyword"), None);
        assert_eq!(SolidityLexer::match_keyword(b"x"), None);
    }

    #[test]
    fn test_solidity_comments_and_strings() {
        let code = b"// SPDX-License-Identifier: MIT\n\
                     pragma solidity ^0.8.0;\n\
                     /* multi-line\n\
                        comment */\n\
                     contract Test {\n\
                         string public name = \"hello\";\n\
                     }\n";
        let mut res = ParseResult::default();
        parse_into(code, &mut res);
        assert_eq!(res.lines_processed, 7);
        assert_eq!(res.comment_lines, 3);
        assert_eq!(res.code_lines, 4);
        assert_eq!(res.blank_lines, 0);
        assert!(res.total_keywords >= 4);
    }
}
