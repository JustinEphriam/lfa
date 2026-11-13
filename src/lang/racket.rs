use super::{
    compute_keyword_lens, compute_kw_max_len, compute_kw_min_len, count_newlines_avx2,
    find_byte_avx2, make_token_lut_lo, skip_literal, unified_parse_into, LangLexer,
    ParseResult, STATE_CODE, STATE_COMMENT, STATE_NONE, TOK_CODE, TOK_NL, TOK_WS,
};

pub const KEYWORD_COUNT: usize = 15;
const KW_SLOT_MASK: usize = 31;
const KW_EMPTY: u8 = 255;

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

const _: () = assert!(KEYWORD_NAMES.len() == KEYWORD_COUNT);
const _: () = assert!(KEYWORD_LENS.len() == KEYWORD_COUNT);
const _: () = assert!(KW_MIN_LEN >= 2 && KW_MAX_LEN <= 24);

#[inline(always)]
const fn kw_hash(len: usize, first: u8, mid: u8, last: u8) -> usize {
    let val = len.wrapping_mul(1395)
        .wrapping_add((first as usize).wrapping_mul(9604))
        .wrapping_add((mid as usize).wrapping_mul(5944))
        .wrapping_add((last as usize).wrapping_mul(8434));
    (val ^ (val >> 1)) & KW_SLOT_MASK
}

const KW_TABLE: [u8; 32] = {
    let mut table = [KW_EMPTY; 32];
    let mut i = 0;
    while i < KEYWORD_COUNT {
        let name = KEYWORD_NAMES[i].as_bytes();
        let len = KEYWORD_LENS[i] as usize;
        let h = kw_hash(len, name[0], name[len / 2], name[len - 1]);
        assert!(table[h] == KW_EMPTY);
        table[h] = i as u8;
        i += 1;
    }
    table
};

const TOK_IDENT: u8 = 3;
const TOK_SEMICOLON: u8 = 4;
const TOK_HASH: u8 = 5;
const TOK_QUOTE: u8 = 6;

pub const TOKEN_KIND: [u8; 256] = {
    let mut t = [TOK_CODE; 256];
    t[b' ' as usize] = TOK_WS;
    t[b'\t' as usize] = TOK_WS;
    t[b'\r' as usize] = TOK_WS;
    t[0x0b_usize] = TOK_WS;
    t[0x0c_usize] = TOK_WS;
    t[b'\n' as usize] = TOK_NL;

    let mut c = b'a';
    while c <= b'z' {
        t[c as usize] = TOK_IDENT;
        c += 1;
    }
    let mut c = b'A';
    while c <= b'Z' {
        t[c as usize] = TOK_IDENT;
        c += 1;
    }
    t[b'_' as usize] = TOK_IDENT;
    t[b'-' as usize] = TOK_IDENT;
    t[b'?' as usize] = TOK_IDENT;
    t[b'!' as usize] = TOK_IDENT;
    t[b';' as usize] = TOK_SEMICOLON;
    t[b'#' as usize] = TOK_HASH;
    t[b'"' as usize] = TOK_QUOTE;
    t
};

pub const TOKEN_LUT_LO: [i8; 32] = make_token_lut_lo(&TOKEN_KIND);

#[inline(always)]
const fn len_matches(n: usize, idx: usize) -> bool {
    n == KEYWORD_LENS[idx] as usize
}

#[inline(always)]
fn skip_racket_block_comment(content: &[u8], start_i: usize) -> usize {
    let content_len = content.len();
    let mut depth = 1usize;
    let mut i = start_i;
    while i + 1 < content_len {
        let b0 = content[i];
        let b1 = content[i + 1];
        if b0 == b'#' && b1 == b'|' {
            depth += 1;
            i += 2;
        } else if b0 == b'|' && b1 == b'#' {
            depth -= 1;
            i += 2;
            if depth == 0 {
                return i;
            }
        } else {
            i += 1;
        }
    }
    content_len
}

pub struct RacketLexer;

impl LangLexer<KEYWORD_COUNT> for RacketLexer {
    const TOKEN_KIND: [u8; 256] = TOKEN_KIND;
    const TOKEN_LUT_LO: [i8; 32] = TOKEN_LUT_LO;
    const HAS_DOLLAR_IDENT: bool = false;
    const SUPPORTS_TEMPLATES: bool = false;

    #[inline(always)]
    fn match_keyword(cand: &[u8]) -> Option<usize> {
        let n = cand.len();
        if n.wrapping_sub(KW_MIN_LEN) <= KW_MAX_LEN - KW_MIN_LEN {
            let h = kw_hash(n, cand[0], cand[n / 2], cand[n - 1]);
            let slot = KW_TABLE[h];
            if slot != KW_EMPTY {
                let idx = slot as usize;
                if len_matches(n, idx) && cand == KEYWORD_NAMES[idx].as_bytes() {
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
        let content_len = content.len();
        match kind {
            TOK_SEMICOLON => {
                *line_state |= STATE_COMMENT;
                if let Some(pos) = find_byte_avx2(&content[i + 1..], b'\n') {
                    if (*line_state & STATE_CODE) != 0 {
                        acc.code_lines += 1;
                    } else {
                        acc.comment_lines += 1;
                    }
                    acc.lines_processed += 1;
                    *line_state = STATE_NONE;
                    i + 1 + pos + 1
                } else {
                    content_len
                }
            }
            TOK_HASH => {
                if i + 1 < content_len && content[i + 1] == b'|' {
                    *line_state |= STATE_COMMENT;
                    let end_i = skip_racket_block_comment(content, i + 2);
                    let nl = count_newlines_avx2(&content[i + 2..end_i]);
                    if nl > 0 {
                        if (*line_state & STATE_CODE) != 0 {
                            acc.code_lines += 1;
                            acc.comment_lines += (nl - 1) as i64;
                        } else {
                            acc.comment_lines += nl as i64;
                        }
                        acc.lines_processed += nl as i64;
                        *line_state = STATE_COMMENT;
                    }
                    end_i
                } else if i + 1 < content_len && content[i + 1] == b';' {
                    *line_state |= STATE_COMMENT;
                    i + 2
                } else {
                    *line_state |= STATE_CODE;
                    i + 1
                }
            }
            TOK_QUOTE => {
                *line_state |= STATE_CODE;
                let end_i = skip_literal(content, i + 1, b'"');
                let nl = count_newlines_avx2(&content[i + 1..end_i]);
                if nl > 0 {
                    acc.code_lines += nl as i64;
                    acc.lines_processed += nl as i64;
                }
                end_i
            }
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
