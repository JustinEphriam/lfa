use super::{
    compute_keyword_lens, compute_kw_max_len, compute_kw_min_len, count_newlines_avx2,
    finish_line_comment, handle_quoted_literal, make_token_kind, make_token_lut_lo,
    match_keyword_with, unified_parse_into, LangLexer, ParseResult, STATE_CODE,
};

pub const KEYWORD_COUNT: usize = 49;
const KW_SLOT_MASK: usize = 127;
const KW_EMPTY: u8 = 255;

pub const KEYWORD_NAMES: [&str; KEYWORD_COUNT] = [
    "abs",
    "add",
    "and",
    "atan",
    "bind",
    "ceiling",
    "clear",
    "copy",
    "cos",
    "count",
    "def",
    "div",
    "dup",
    "eq",
    "exch",
    "exec",
    "exit",
    "exp",
    "floor",
    "for",
    "ge",
    "gt",
    "idiv",
    "if",
    "ifelse",
    "index",
    "le",
    "ln",
    "log",
    "loop",
    "lt",
    "mod",
    "mul",
    "ne",
    "neg",
    "not",
    "or",
    "pop",
    "repeat",
    "roll",
    "round",
    "showpage",
    "sin",
    "sqrt",
    "stop",
    "stopped",
    "sub",
    "truncate",
    "xor",
];

pub const KEYWORD_LENS: [u8; KEYWORD_COUNT] = compute_keyword_lens(&KEYWORD_NAMES);
pub const KW_MIN_LEN: usize = compute_kw_min_len(&KEYWORD_NAMES);
pub const KW_MAX_LEN: usize = compute_kw_max_len(&KEYWORD_NAMES);

const _: () = assert!(KEYWORD_NAMES.len() == KEYWORD_COUNT);
const _: () = assert!(KEYWORD_LENS.len() == KEYWORD_COUNT);
const _: () = assert!(KW_MIN_LEN >= 2 && KW_MAX_LEN <= 8);

#[inline(always)]
const fn kw_hash(cand: &[u8], len: usize) -> usize {
    let b0 = cand[0] as usize;
    let bmid = cand[len / 2] as usize;
    let blast = cand[len - 1] as usize;
    let val = len.wrapping_mul(8671)
        .wrapping_add(b0.wrapping_mul(7875))
        .wrapping_add(bmid.wrapping_mul(2223))
        .wrapping_add(blast.wrapping_mul(4850));
    (val ^ (val >> 2)) & KW_SLOT_MASK
}

const KW_TABLE: [u8; 128] = {
    let mut table = [KW_EMPTY; 128];
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
const TOK_PERCENT: u8 = 4;
const TOK_QUOTE: u8 = 5;
const TOK_LPAREN: u8 = 6;

pub const TOKEN_KIND: [u8; 256] = make_token_kind(
    TOK_IDENT,
    &[
        (b'%', TOK_PERCENT),
        (b'"', TOK_QUOTE),
        (b'(', TOK_LPAREN),
    ],
);

pub const TOKEN_LUT_LO: [i8; 32] = make_token_lut_lo(&TOKEN_KIND);

#[inline(always)]
fn handle_postscript_paren(
    content: &[u8],
    i: usize,
    line_state: &mut u8,
    acc: &mut ParseResult<KEYWORD_COUNT>,
) -> usize {
    *line_state |= STATE_CODE;
    let content_len = content.len();
    let mut depth = 1usize;
    let mut p = i + 1;
    while p < content_len {
        let b = content[p];
        if b == b'\\' {
            p += 2;
        } else if b == b'(' {
            depth += 1;
            p += 1;
        } else if b == b')' {
            depth -= 1;
            p += 1;
            if depth == 0 {
                break;
            }
        } else {
            p += 1;
        }
    }
    let end_i = p.min(content_len);
    let nl = count_newlines_avx2(&content[i + 1..end_i]);
    if nl > 0 {
        acc.code_lines += nl as i64;
        acc.lines_processed += nl as i64;
    }
    end_i
}

#[allow(dead_code)]
#[cfg(test)]
#[inline(always)]
pub fn is_postscript_extension(name: &[u8]) -> bool {
    super::detect_language(name, super::LANG_ALL_MASK) == Some(super::LangId::PostScript)
}

pub struct PostScriptLexer;

impl LangLexer<KEYWORD_COUNT> for PostScriptLexer {
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
            TOK_PERCENT => finish_line_comment(content, i + 1, line_state, acc),
            TOK_QUOTE => handle_quoted_literal(content, i, b'"', line_state, acc),
            TOK_LPAREN => handle_postscript_paren(content, i, line_state, acc),
            _ => {
                *line_state |= STATE_CODE;
                i + 1
            }
        }
    }
}

pub fn parse_into(content: &[u8], acc: &mut ParseResult<KEYWORD_COUNT>) {
    unified_parse_into::<KEYWORD_COUNT, PostScriptLexer>(content, acc);
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_postscript_keyword_table_and_mph() {
        for (i, &name) in KEYWORD_NAMES.iter().enumerate() {
            let idx = PostScriptLexer::match_keyword(name.as_bytes());
            assert_eq!(idx, Some(i), "keyword {} failed match", name);
        }
        assert_eq!(PostScriptLexer::match_keyword(b"notakeyword"), None);
        assert_eq!(PostScriptLexer::match_keyword(b"x"), None);
    }

    #[test]
    fn test_postscript_comments_and_strings() {
        let code = b"%!PS\n\
                     /Helvetica findfont 12 scalefont setfont\n\
                     (Hello, world!) show\n\
                     showpage\n";
        let mut res = ParseResult::default();
        parse_into(code, &mut res);
        assert_eq!(res.lines_processed, 4);
        assert_eq!(res.blank_lines, 0);
        assert_eq!(res.comment_lines, 1);
        assert_eq!(res.code_lines, 3);
    }
}
