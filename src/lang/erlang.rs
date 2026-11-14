use super::{
    compute_keyword_lens, compute_kw_max_len, compute_kw_min_len, finish_line_comment,
    handle_quoted_literal, kw_match_2_to_16, make_token_kind, make_token_lut_lo,
    unified_parse_into, LangLexer, ParseResult, STATE_CODE,
};

pub const KEYWORD_COUNT: usize = 27;
const KW_SLOT_MASK: usize = 63;
const KW_EMPTY: u8 = 255;

pub const KEYWORD_NAMES: [&str; KEYWORD_COUNT] = [
    "after",
    "and",
    "andalso",
    "band",
    "begin",
    "bnot",
    "bor",
    "bsl",
    "bsr",
    "bxor",
    "case",
    "catch",
    "cond",
    "div",
    "end",
    "fun",
    "if",
    "let",
    "not",
    "of",
    "or",
    "orelse",
    "receive",
    "rem",
    "try",
    "when",
    "xor",
];

pub const KEYWORD_LENS: [u8; KEYWORD_COUNT] = compute_keyword_lens(&KEYWORD_NAMES);
pub const KW_MIN_LEN: usize = compute_kw_min_len(&KEYWORD_NAMES);
pub const KW_MAX_LEN: usize = compute_kw_max_len(&KEYWORD_NAMES);

const _: () = assert!(KEYWORD_NAMES.len() == KEYWORD_COUNT);
const _: () = assert!(KEYWORD_LENS.len() == KEYWORD_COUNT);

#[inline(always)]
const fn kw_hash(first: u8, mid: u8, last: u8, len: usize) -> usize {
    let val = (first as usize)
        .wrapping_mul(142)
        .wrapping_add((mid as usize).wrapping_mul(90))
        .wrapping_add((last as usize).wrapping_mul(89))
        .wrapping_add(len.wrapping_mul(42));
    (val ^ (val >> 3)) & KW_SLOT_MASK
}

const KW_TABLE: [u8; 64] = {
    let mut table = [KW_EMPTY; 64];
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

#[inline(always)]
const fn len_matches(n: usize, idx: usize) -> bool {
    n == KEYWORD_LENS[idx] as usize
}

#[cfg(test)]
#[inline(always)]
pub fn is_erlang_extension(name: &[u8]) -> bool {
    super::detect_language(name, super::LANG_ALL_MASK) == Some(super::LangId::Erlang)
}

pub struct ErlangLexer;

impl LangLexer<KEYWORD_COUNT> for ErlangLexer {
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
        match kind {
            TOK_PERCENT => finish_line_comment(content, i + 1, line_state, acc),
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
    unified_parse_into::<KEYWORD_COUNT, ErlangLexer>(content, acc);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_erlang_extensions() {
        assert!(is_erlang_extension(b"server.erl"));
        assert!(is_erlang_extension(b"header.hrl"));
        assert!(!is_erlang_extension(b"main.rs"));
        assert!(!is_erlang_extension(b"main.c"));
    }

    #[test]
    fn test_erlang_keyword_table_and_mph() {
        for (i, name) in KEYWORD_NAMES.iter().enumerate() {
            let b = name.as_bytes();
            let h = kw_hash(b[0], b[b.len() / 2], b[b.len() - 1], b.len());
            assert_eq!(KW_TABLE[h], i as u8);
            assert_eq!(ErlangLexer::match_keyword(b), Some(i));
        }
        assert_eq!(ErlangLexer::match_keyword(b"notakeyword"), None);
        assert_eq!(ErlangLexer::match_keyword(b"erl"), None);
    }

    #[test]
    fn test_erlang_comments_and_strings() {
        let src = b"% comment\n-module(math).\n-export([add/2]).\n\nadd(A, B) ->\n    io:format(\"adding ~p and ~p~n\", [A, B]),\n    case A of\n        0 -> B;\n        _ -> A + B\n    end.\n";
        let mut acc = ParseResult::default();
        parse_into(src, &mut acc);
        assert_eq!(acc.comment_lines, 1);
        assert_eq!(acc.code_lines, 8);
        assert_eq!(acc.blank_lines, 1);
        assert!(acc.total_keywords >= 3);
    }
}
