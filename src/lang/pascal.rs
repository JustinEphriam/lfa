use super::{
    compute_keyword_lens, compute_kw_max_len, compute_kw_min_len, count_newlines_avx2,
    find_byte_avx2, make_token_lut_lo, skip_literal, unified_parse_into, LangLexer,
    ParseResult, STATE_CODE, STATE_COMMENT, STATE_NONE, TOK_CODE, TOK_NL, TOK_WS,
};

pub const KEYWORD_COUNT: usize = 65;
const KW_SLOT_MASK: usize = 255;
const KW_EMPTY: u8 = 255;

pub const KEYWORD_NAMES: [&str; KEYWORD_COUNT] = [
    "and", "array", "as", "asm", "begin", "case", "class", "const", "constructor",
    "destructor", "div", "do", "downto", "else", "end", "except", "exports",
    "file", "finalization", "finally", "for", "function", "goto", "if",
    "implementation", "in", "inherited", "initialization", "inline", "interface",
    "is", "label", "library", "mod", "nil", "not", "object", "of", "on",
    "operator", "or", "packed", "procedure", "program", "property", "raise",
    "record", "repeat", "resourcestring", "set", "shl", "shr", "string",
    "then", "threadvar", "to", "try", "type", "unit", "until", "uses",
    "var", "while", "with", "xor",
];

pub const KEYWORD_LENS: [u8; KEYWORD_COUNT] = compute_keyword_lens(&KEYWORD_NAMES);
pub const KW_MIN_LEN: usize = compute_kw_min_len(&KEYWORD_NAMES);
pub const KW_MAX_LEN: usize = compute_kw_max_len(&KEYWORD_NAMES);

const _: () = assert!(KEYWORD_NAMES.len() == KEYWORD_COUNT);
const _: () = assert!(KEYWORD_LENS.len() == KEYWORD_COUNT);
const _: () = assert!(KW_MIN_LEN >= 2 && KW_MAX_LEN <= 16);

#[inline(always)]
const fn kw_hash(first: u8, mid: u8, last: u8, len: usize) -> usize {
    let val = (first as usize).wrapping_mul(1)
        ^ (mid as usize).wrapping_mul(11)
        ^ (last as usize).wrapping_mul(34)
        ^ len.wrapping_mul(17);
    val & KW_SLOT_MASK
}

const KW_TABLE: [u8; 256] = {
    let mut table = [KW_EMPTY; 256];
    let mut i = 0;
    while i < KEYWORD_COUNT {
        let name = KEYWORD_NAMES[i].as_bytes();
        let len = KEYWORD_LENS[i] as usize;
        let h = kw_hash(name[0], name[len / 2], name[len - 1], len);
        assert!(table[h] == KW_EMPTY);
        table[h] = i as u8;
        i += 1;
    }
    table
};

const TOK_IDENT: u8 = 3;
const TOK_SLASH: u8 = 4;
const TOK_LBRACE: u8 = 5;
const TOK_LPAREN: u8 = 6;
const TOK_SQUOTE: u8 = 7;

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
    t[b'/' as usize] = TOK_SLASH;
    t[b'{' as usize] = TOK_LBRACE;
    t[b'(' as usize] = TOK_LPAREN;
    t[b'\'' as usize] = TOK_SQUOTE;
    t
};

pub const TOKEN_LUT_LO: [i8; 32] = make_token_lut_lo(&TOKEN_KIND);

#[inline(always)]
const fn len_matches(n: usize, idx: usize) -> bool {
    n == KEYWORD_LENS[idx] as usize
}

pub struct PascalLexer;

impl LangLexer<KEYWORD_COUNT> for PascalLexer {
    const TOKEN_KIND: [u8; 256] = TOKEN_KIND;
    const TOKEN_LUT_LO: [i8; 32] = TOKEN_LUT_LO;
    const HAS_DOLLAR_IDENT: bool = false;
    const SUPPORTS_TEMPLATES: bool = false;

    #[inline(always)]
    fn match_keyword(cand: &[u8]) -> Option<usize> {
        let n = cand.len();
        if n.wrapping_sub(KW_MIN_LEN) <= KW_MAX_LEN - KW_MIN_LEN {
            let first = cand[0] | 0x20;
            let mid = cand[n / 2] | 0x20;
            let last = cand[n - 1] | 0x20;
            let h = kw_hash(first, mid, last, n);
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
            TOK_SLASH => {
                if i + 1 < content_len && content[i + 1] == b'/' {
                    *line_state |= STATE_COMMENT;
                    if let Some(pos) = find_byte_avx2(&content[i + 2..], b'\n') {
                        if (*line_state & STATE_CODE) != 0 {
                            acc.code_lines += 1;
                        } else {
                            acc.comment_lines += 1;
                        }
                        acc.lines_processed += 1;
                        *line_state = STATE_NONE;
                        i + 2 + pos + 1
                    } else {
                        content_len
                    }
                } else {
                    *line_state |= STATE_CODE;
                    i + 1
                }
            }
            TOK_LBRACE => {
                *line_state |= STATE_COMMENT;
                if let Some(pos) = find_byte_avx2(&content[i + 1..], b'}') {
                    let end = i + 1 + pos;
                    let nl = count_newlines_avx2(&content[i + 1..end]);
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
                    end + 1
                } else {
                    let nl = count_newlines_avx2(&content[i + 1..]);
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
                    content_len
                }
            }
            TOK_LPAREN => {
                if i + 1 < content_len && content[i + 1] == b'*' {
                    *line_state |= STATE_COMMENT;
                    let mut cur = i + 2;
                    while cur < content_len {
                        if cur + 1 < content_len && content[cur] == b'*' && content[cur + 1] == b')' {
                            let end = cur;
                            let nl = count_newlines_avx2(&content[i + 2..end]);
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
                            return cur + 2;
                        }
                        cur += 1;
                    }
                    let nl = count_newlines_avx2(&content[i + 2..]);
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
                    content_len
                } else {
                    *line_state |= STATE_CODE;
                    i + 1
                }
            }
            TOK_SQUOTE => {
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

pub fn parse_into(content: &[u8], res: &mut ParseResult<KEYWORD_COUNT>) {
    unified_parse_into::<KEYWORD_COUNT, PascalLexer>(content, res);
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_pascal_keyword_table_and_mph() {
        for (i, &name) in KEYWORD_NAMES.iter().enumerate() {
            let idx = PascalLexer::match_keyword(name.as_bytes());
            assert_eq!(idx, Some(i), "keyword {} failed match", name);
            let upper = name.to_ascii_uppercase();
            let idx_upper = PascalLexer::match_keyword(upper.as_bytes());
            assert_eq!(idx_upper, Some(i), "keyword {} upper failed match", upper);
        }
        assert_eq!(PascalLexer::match_keyword(b"notakeyword"), None);
        assert_eq!(PascalLexer::match_keyword(b"x"), None);
    }

    #[test]
    fn test_pascal_comments_and_strings() {
        let code = b"// Pascal program\n                     program Hello;\n                     begin\n                         WriteLn('Hello World'); { inline comment }\n                         (* multi\n                            line *)\n                     end.\n";
        let mut res = ParseResult::default();
        parse_into(code, &mut res);
        assert_eq!(res.lines_processed, 7);
        assert_eq!(res.comment_lines, 3);
        assert_eq!(res.code_lines, 4);
        assert_eq!(res.blank_lines, 0);
        assert!(res.total_keywords >= 2);
    }
}
