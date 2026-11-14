use super::{
    find_star_slash, count_newlines_avx2,
    compute_keyword_lens, compute_kw_max_len, compute_kw_min_len, find_byte_avx2, make_token_lut_lo, skip_literal, unified_parse_into,
    LangLexer, ParseResult, STATE_CODE, STATE_COMMENT, TOK_CODE, TOK_NL, TOK_WS,
};

pub const KEYWORD_COUNT: usize = 21;
const KW_SLOT_MASK: usize = 31;
const KW_EMPTY: u8 = 255;

pub const KEYWORD_NAMES: [&str; KEYWORD_COUNT] = [
    "break",
    "case",
    "catch",
    "class",
    "continue",
    "critical",
    "default",
    "else",
    "finally",
    "for",
    "global",
    "goto",
    "if",
    "loop",
    "return",
    "static",
    "switch",
    "throw",
    "try",
    "until",
    "while",
];

pub const KEYWORD_LENS: [u8; KEYWORD_COUNT] = compute_keyword_lens(&KEYWORD_NAMES);
pub const KW_MIN_LEN: usize = compute_kw_min_len(&KEYWORD_NAMES);
pub const KW_MAX_LEN: usize = compute_kw_max_len(&KEYWORD_NAMES);

const _: () = assert!(KEYWORD_NAMES.len() == KEYWORD_COUNT);
const _: () = assert!(KEYWORD_LENS.len() == KEYWORD_COUNT);
const _: () = assert!(KW_MIN_LEN >= 2 && KW_MAX_LEN <= 16);

const TOK_IDENT: u8 = 3;
const TOK_SEMI: u8 = 4;
const TOK_SLASH: u8 = 5;
const TOK_QUOTE: u8 = 6;
const TOK_APOSTROPHE: u8 = 7;

#[inline(always)]
const fn kw_hash(cand: &[u8], len: usize) -> usize {
    let b0 = (cand[0] | 0x20) as usize;
    let b1 = (cand[1] | 0x20) as usize;
    let bmid = (cand[len / 2] | 0x20) as usize;
    let blast = (cand[len - 1] | 0x20) as usize;
    let val = len.wrapping_mul(7682)
        .wrapping_add(b0.wrapping_mul(8425))
        .wrapping_add(bmid.wrapping_mul(6726))
        .wrapping_add(blast.wrapping_mul(4109))
        .wrapping_add(b1.wrapping_mul(1276));
    (val ^ (val >> 2)) & KW_SLOT_MASK
}

const KW_TABLE: [u8; 32] = {
    let mut table = [KW_EMPTY; 32];
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

pub const TOKEN_KIND: [u8; 256] = {
    let mut table = [TOK_CODE; 256];
    table[b' ' as usize] = TOK_WS;
    table[b'\t' as usize] = TOK_WS;
    table[b'\r' as usize] = TOK_WS;
    table[0x0b_usize] = TOK_WS;
    table[0x0c_usize] = TOK_WS;
    table[b'\n' as usize] = TOK_NL;
    let mut c = b'a';
    while c <= b'z' {
        table[c as usize] = TOK_IDENT;
        c += 1;
    }
    let mut c = b'A';
    while c <= b'Z' {
        table[c as usize] = TOK_IDENT;
        c += 1;
    }
    table[b'_' as usize] = TOK_IDENT;
    table[b';' as usize] = TOK_SEMI;
    table[b'/' as usize] = TOK_SLASH;
    table[b'"' as usize] = TOK_QUOTE;
    table[b'\'' as usize] = TOK_APOSTROPHE;
    table
};

pub const TOKEN_LUT_LO: [i8; 32] = make_token_lut_lo(&TOKEN_KIND);

#[inline(always)]
const fn len_matches(n: usize, idx: usize) -> bool {
    n == KEYWORD_LENS[idx] as usize
}

pub struct AutoHotkeyLexer;

impl LangLexer<KEYWORD_COUNT> for AutoHotkeyLexer {
    const TOKEN_KIND: [u8; 256] = TOKEN_KIND;
    const TOKEN_LUT_LO: [i8; 32] = TOKEN_LUT_LO;
    const HAS_DOLLAR_IDENT: bool = false;
    const SUPPORTS_TEMPLATES: bool = false;

    #[inline(always)]
    fn match_keyword(cand: &[u8]) -> Option<usize> {
        let n = cand.len();
        if n.wrapping_sub(KW_MIN_LEN) <= KW_MAX_LEN - KW_MIN_LEN {
            let h = kw_hash(cand, n);
            let slot = KW_TABLE[h];
            if slot != KW_EMPTY {
                let idx = slot as usize;
                if len_matches(n, idx) {
                    let kw = KEYWORD_NAMES[idx].as_bytes();
                    let mut i = 0;
                    while i < n {
                        if (cand[i] | 0x20) != kw[i] {
                            return None;
                        }
                        i += 1;
                    }
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
            TOK_SEMI => {
                *line_state |= STATE_COMMENT;
                if let Some(pos) = find_byte_avx2(&content[i + 1..], b'\n') {
                    if (*line_state & STATE_CODE) != 0 {
                        acc.code_lines += 1;
                    } else {
                        acc.comment_lines += 1;
                    }
                    acc.lines_processed += 1;
                    *line_state = 0;
                    i + 1 + pos + 1
                } else {
                    content_len
                }
            }
            TOK_SLASH => {
                if i + 1 < content_len && content[i + 1] == b'*' {
                    *line_state |= STATE_COMMENT;
                    if let Some(pos) = find_star_slash(&content[i + 2..]) {
                        let star_slash_pos = i + 2 + pos;
                        if let Some(first_nl) = find_byte_avx2(&content[i + 2..star_slash_pos], b'\n') {
                            if (*line_state & STATE_CODE) != 0 {
                                acc.code_lines += 1;
                            } else {
                                acc.comment_lines += 1;
                            }
                            acc.lines_processed += 1;
                            let first_nl_abs = i + 2 + first_nl;
                            let inter_nls = count_newlines_avx2(&content[first_nl_abs + 1..star_slash_pos]);
                            acc.comment_lines += inter_nls as i64;
                            acc.lines_processed += inter_nls as i64;
                            *line_state = STATE_COMMENT;
                        }
                        return star_slash_pos + 2;
                    } else {
                        if let Some(first_nl) = find_byte_avx2(&content[i + 2..], b'\n') {
                            if (*line_state & STATE_CODE) != 0 {
                                acc.code_lines += 1;
                            } else {
                                acc.comment_lines += 1;
                            }
                            acc.lines_processed += 1;
                            let first_nl_abs = i + 2 + first_nl;
                            let rem_nls = count_newlines_avx2(&content[first_nl_abs + 1..]);
                            acc.comment_lines += rem_nls as i64;
                            acc.lines_processed += rem_nls as i64;
                            *line_state = STATE_COMMENT;
                        }
                        return content_len;
                    }
                }
                *line_state |= STATE_CODE;
                i + 1
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
            TOK_APOSTROPHE => {
                *line_state |= STATE_CODE;
                let end_i = skip_literal(content, i + 1, b'\'');
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

#[inline(always)]
pub fn parse_into(content: &[u8], acc: &mut ParseResult<KEYWORD_COUNT>) {
    unified_parse_into::<KEYWORD_COUNT, AutoHotkeyLexer>(content, acc);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keywords() {
        for &kw in KEYWORD_NAMES.iter() {
            let mut acc = ParseResult::default();
            parse_into(kw.as_bytes(), &mut acc);
            assert_eq!(acc.code_lines, 1);
            let idx = AutoHotkeyLexer::match_keyword(kw.as_bytes()).unwrap();
            assert_eq!(acc.keyword_counts[idx], 1);
        }
    }

    #[test]
    fn test_comments() {
        let text = b"; comment\n/* block */\ncode\n";
        let mut acc = ParseResult::default();
        parse_into(text, &mut acc);
        assert_eq!(acc.lines_processed, 3);
        assert_eq!(acc.comment_lines, 2);
        assert_eq!(acc.code_lines, 1);
    }
}
