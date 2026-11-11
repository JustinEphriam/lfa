use super::{
    compute_keyword_lens, compute_kw_max_len, compute_kw_min_len, count_newlines_avx2,
    find_byte_avx2, find_star_slash, kw_match_2_to_16, make_token_lut_lo, skip_literal,
    unified_parse_into, LangLexer, ParseResult, STATE_CODE, STATE_COMMENT, STATE_NONE, TOK_CODE,
    TOK_NL, TOK_WS,
};

pub const KEYWORD_COUNT: usize = 26;
const KW_SLOT_MASK: usize = 255;
const KW_EMPTY: u8 = 255;

pub const KEYWORD_NAMES: [&str; KEYWORD_COUNT] = [
    "alias",
    "break",
    "case",
    "const",
    "const_assert",
    "continue",
    "continuing",
    "default",
    "diagnostic",
    "discard",
    "else",
    "enable",
    "false",
    "fn",
    "for",
    "if",
    "let",
    "loop",
    "override",
    "requires",
    "return",
    "struct",
    "switch",
    "true",
    "var",
    "while",
];

pub const KEYWORD_LENS: [u8; KEYWORD_COUNT] = compute_keyword_lens(&KEYWORD_NAMES);
pub const KW_MIN_LEN: usize = compute_kw_min_len(&KEYWORD_NAMES);
pub const KW_MAX_LEN: usize = compute_kw_max_len(&KEYWORD_NAMES);

const _: () = assert!(KEYWORD_NAMES.len() == KEYWORD_COUNT);
const _: () = assert!(KEYWORD_LENS.len() == KEYWORD_COUNT);
const _: () = assert!(KW_MIN_LEN >= 2 && KW_MAX_LEN <= 16);

#[inline(always)]
const fn kw_hash(first: u8, mid: u8, last: u8, len: usize) -> usize {
    let val = (first as usize)
        .wrapping_mul(2397)
        .wrapping_add((mid as usize).wrapping_mul(6448))
        .wrapping_add((last as usize).wrapping_mul(3598))
        .wrapping_add(len.wrapping_mul(8546));
    (val ^ (val >> 2)) & KW_SLOT_MASK
}

const KW_TABLE: [u8; 256] = {
    let mut table = [KW_EMPTY; 256];
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
const TOK_SLASH: u8 = 4;
const TOK_QUOTE: u8 = 5;

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
    t[b'"' as usize] = TOK_QUOTE;
    t
};

pub const TOKEN_LUT_LO: [i8; 32] = make_token_lut_lo(&TOKEN_KIND);

#[inline(always)]
const fn len_matches(n: usize, idx: usize) -> bool {
    n == KEYWORD_LENS[idx] as usize
}

pub struct WgslLexer;

impl LangLexer<KEYWORD_COUNT> for WgslLexer {
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
        let content_len = content.len();
        match kind {
            TOK_SLASH => {
                if i + 1 < content_len {
                    let next_b = content[i + 1];
                    if next_b == b'/' {
                        *line_state |= STATE_COMMENT;
                        let mut j = i + 2;
                        while j < content_len {
                            let Some(pos) = find_byte_avx2(&content[j..], b'\n') else {
                                j = content_len;
                                break;
                            };
                            let nl_idx = j + pos;
                            let mut check = nl_idx;
                            if check > j && content[check - 1] == b'\r' {
                                check -= 1;
                            }
                            let is_spliced = check > j && content[check - 1] == b'\\';
                            if (*line_state & STATE_CODE) != 0 {
                                acc.code_lines += 1;
                            } else if (*line_state & STATE_COMMENT) != 0 {
                                acc.comment_lines += 1;
                            } else {
                                acc.blank_lines += 1;
                            }
                            acc.lines_processed += 1;
                            if is_spliced {
                                *line_state = STATE_COMMENT;
                                j = nl_idx + 1;
                            } else {
                                *line_state = STATE_NONE;
                                j = nl_idx + 1;
                                break;
                            }
                        }
                        return j;
                    } else if next_b == b'*' {
                        *line_state |= STATE_COMMENT;
                        if let Some(pos) = find_star_slash(&content[i + 2..]) {
                            let star_slash_pos = i + 2 + pos;
                            if let Some(first_nl) =
                                find_byte_avx2(&content[i + 2..star_slash_pos], b'\n')
                            {
                                if (*line_state & STATE_CODE) != 0 {
                                    acc.code_lines += 1;
                                } else {
                                    acc.comment_lines += 1;
                                }
                                acc.lines_processed += 1;
                                let first_nl_abs = i + 2 + first_nl;
                                let inter_nls = count_newlines_avx2(
                                    &content[first_nl_abs + 1..star_slash_pos],
                                );
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
            _ => {
                *line_state |= STATE_CODE;
                i + 1
            }
        }
    }
}

pub fn parse_into(content: &[u8], res: &mut ParseResult<KEYWORD_COUNT>) {
    unified_parse_into::<KEYWORD_COUNT, WgslLexer>(content, res);
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_wgsl_keyword_table_and_mph() {
        for (i, &name) in KEYWORD_NAMES.iter().enumerate() {
            let idx = WgslLexer::match_keyword(name.as_bytes());
            assert_eq!(idx, Some(i), "keyword {} failed match", name);
        }
        assert_eq!(WgslLexer::match_keyword(b"notakeyword"), None);
        assert_eq!(WgslLexer::match_keyword(b"x"), None);
    }

    #[test]
    fn test_wgsl_comments_and_strings() {
        let code = b"// WGSL vertex shader\n\
                     @vertex\n\
                     fn vs_main() -> @builtin(position) vec4<f32> {\n\
                         /* return position */\n\
                         var pos = vec4<f32>(0.0, 0.0, 0.0, 1.0);\n\
                         return pos;\n\
                     }\n";
        let mut res = ParseResult::default();
        parse_into(code, &mut res);
        assert_eq!(res.lines_processed, 7);
        assert_eq!(res.comment_lines, 2);
        assert_eq!(res.code_lines, 5);
        assert_eq!(res.blank_lines, 0);
        assert!(res.total_keywords >= 3);
    }
}
