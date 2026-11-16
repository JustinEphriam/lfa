use super::{
    compute_keyword_lens, compute_kw_max_len, compute_kw_min_len, finish_line_comment,
    handle_nested_2byte_comment, handle_quoted_literal, make_token_kind, make_token_lut_lo,
    match_keyword_caseless_with, unified_parse_into, LangLexer, ParseResult, STATE_CODE,
};

pub const KEYWORD_COUNT: usize = 37;
const KW_SLOT_MASK: usize = 63;
const KW_EMPTY: u8 = 255;

pub const KEYWORD_NAMES: [&str; KEYWORD_COUNT] = [
    "about",
    "after",
    "and",
    "as",
    "at",
    "before",
    "beginning",
    "but",
    "by",
    "div",
    "else",
    "end",
    "error",
    "every",
    "false",
    "first",
    "from",
    "get",
    "if",
    "in",
    "is",
    "it",
    "mod",
    "my",
    "not",
    "of",
    "on",
    "or",
    "repeat",
    "return",
    "set",
    "tell",
    "then",
    "to",
    "true",
    "where",
    "with",
];

pub const KEYWORD_LENS: [u8; KEYWORD_COUNT] = compute_keyword_lens(&KEYWORD_NAMES);
pub const KW_MIN_LEN: usize = compute_kw_min_len(&KEYWORD_NAMES);
pub const KW_MAX_LEN: usize = compute_kw_max_len(&KEYWORD_NAMES);

const _: () = assert!(KEYWORD_NAMES.len() == KEYWORD_COUNT);
const _: () = assert!(KEYWORD_LENS.len() == KEYWORD_COUNT);
const _: () = assert!(KW_MIN_LEN >= 2 && KW_MAX_LEN <= 9);

#[inline(always)]
const fn kw_hash(cand: &[u8], len: usize) -> usize {
    let b0 = (cand[0] | 0x20) as usize;
    let bmid = (cand[len / 2] | 0x20) as usize;
    let blast = (cand[len - 1] | 0x20) as usize;
    let b1 = (cand[1] | 0x20) as usize;
    let val = len.wrapping_mul(4574)
        .wrapping_add(b0.wrapping_mul(6999))
        .wrapping_add(b1.wrapping_mul(9937))
        .wrapping_add(bmid.wrapping_mul(7250))
        .wrapping_add(blast.wrapping_mul(420));
    (val ^ (val >> 1)) & KW_SLOT_MASK
}

const KW_TABLE: [u8; 64] = {
    let mut table = [KW_EMPTY; 64];
    let mut i = 0;
    while i < KEYWORD_COUNT {
        let name = KEYWORD_NAMES[i].as_bytes();
        let len = KEYWORD_LENS[i] as usize;
        assert!(name.len() == len);
        assert!(len >= KW_MIN_LEN && len <= KW_MAX_LEN);
        let h = kw_hash(name, len);
        assert!(table[h] == KW_EMPTY);
        table[h] = i as u8;
        i += 1;
    }
    table
};

const TOK_IDENT: u8 = 3;
const TOK_MINUS: u8 = 4;
const TOK_PAREN: u8 = 5;
const TOK_QUOTE: u8 = 6;

pub const TOKEN_KIND: [u8; 256] = make_token_kind(
    TOK_IDENT,
    &[
        (b'-', TOK_MINUS),
        (b'(', TOK_PAREN),
        (b'"', TOK_QUOTE),
    ],
);

pub const TOKEN_LUT_LO: [i8; 32] = make_token_lut_lo(&TOKEN_KIND);

#[allow(dead_code)]
#[cfg(test)]
#[inline(always)]
pub fn is_applescript_extension(name: &[u8]) -> bool {
    super::detect_language(name, super::LANG_ALL_MASK) == Some(super::LangId::AppleScript)
}

pub struct AppleScriptLexer;

impl LangLexer<KEYWORD_COUNT> for AppleScriptLexer {
    const TOKEN_KIND: [u8; 256] = TOKEN_KIND;
    const TOKEN_LUT_LO: [i8; 32] = TOKEN_LUT_LO;
    const HAS_DOLLAR_IDENT: bool = false;
    const SUPPORTS_TEMPLATES: bool = false;

    #[inline(always)]
    fn match_keyword(cand: &[u8]) -> Option<usize> {
        match_keyword_caseless_with(
            cand,
            KW_MIN_LEN,
            KW_MAX_LEN,
            &KW_TABLE,
            &KEYWORD_LENS,
            &KEYWORD_NAMES,
            |c| kw_hash(c, c.len()),
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
            TOK_MINUS => {
                if i + 1 < content.len() && content[i + 1] == b'-' {
                    finish_line_comment(content, i + 2, line_state, acc)
                } else {
                    *line_state |= STATE_CODE;
                    i + 1
                }
            }
            TOK_PAREN => {
                if i + 1 < content.len() && content[i + 1] == b'*' {
                    handle_nested_2byte_comment(content, i, *b"(*", *b"*)", line_state, acc)
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

pub fn parse_into(content: &[u8], acc: &mut ParseResult<KEYWORD_COUNT>) {
    unified_parse_into::<KEYWORD_COUNT, AppleScriptLexer>(content, acc);
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_applescript_keyword_table_and_mph() {
        for (i, &name) in KEYWORD_NAMES.iter().enumerate() {
            let idx = AppleScriptLexer::match_keyword(name.as_bytes());
            assert_eq!(idx, Some(i), "keyword {} failed match", name);
        }
        assert_eq!(AppleScriptLexer::match_keyword(b"TELL"), Some(31));
        assert_eq!(AppleScriptLexer::match_keyword(b"Return"), Some(29));
        assert_eq!(AppleScriptLexer::match_keyword(b"notakeyword"), None);
        assert_eq!(AppleScriptLexer::match_keyword(b"x"), None);
    }

    #[test]
    fn test_applescript_comments_and_strings() {
        let code = b"-- AppleScript comment\n\
                     tell application \"Finder\" to activate\n\
                     (* block (* nested *) *)\n";
        let mut res = ParseResult::default();
        parse_into(code, &mut res);
        assert_eq!(res.lines_processed, 3);
        assert_eq!(res.blank_lines, 0);
        assert_eq!(res.comment_lines, 2);
        assert_eq!(res.code_lines, 1);
    }
}
