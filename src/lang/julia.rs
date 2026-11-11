use super::{
    compute_keyword_lens, compute_kw_max_len, compute_kw_min_len, count_newlines_avx2,
    find_byte_avx2, kw_match_2_to_16, make_token_lut_lo, skip_literal,
    skip_nested_hash_equal_comment, skip_triple_literal, unified_parse_into, LangLexer,
    ParseResult, STATE_CODE, STATE_COMMENT, STATE_NONE, TOK_CODE, TOK_NL, TOK_WS,
};

pub const KEYWORD_COUNT: usize = 29;
const KW_SLOT_MASK: usize = 63;
const KW_EMPTY: u8 = 255;

pub const KEYWORD_NAMES: [&str; KEYWORD_COUNT] = [
    "baremodule",
    "begin",
    "break",
    "catch",
    "const",
    "continue",
    "do",
    "else",
    "elseif",
    "end",
    "export",
    "finally",
    "for",
    "function",
    "global",
    "if",
    "import",
    "let",
    "local",
    "macro",
    "module",
    "mutable",
    "primitive",
    "quote",
    "return",
    "struct",
    "try",
    "using",
    "while",
];

pub const KEYWORD_LENS: [u8; KEYWORD_COUNT] = compute_keyword_lens(&KEYWORD_NAMES);
pub const KW_MIN_LEN: usize = compute_kw_min_len(&KEYWORD_NAMES);
pub const KW_MAX_LEN: usize = compute_kw_max_len(&KEYWORD_NAMES);

const _: () = assert!(KEYWORD_NAMES.len() == KEYWORD_COUNT);
const _: () = assert!(KEYWORD_LENS.len() == KEYWORD_COUNT);

#[inline(always)]
const fn kw_hash(first: u8, mid: u8, last: u8, len: usize) -> usize {
    let val = (first as usize)
        .wrapping_mul(1)
        .wrapping_add((mid as usize).wrapping_mul(4))
        .wrapping_add((last as usize).wrapping_mul(16))
        .wrapping_add(len.wrapping_mul(154));
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
        let h = kw_hash(name[0], name[len / 2], name[len - 1], len);
        assert!(table[h] == KW_EMPTY);
        table[h] = i as u8;
        i += 1;
    }
    table
};

const TOK_IDENT: u8 = 3;
const TOK_HASH: u8 = 4;
const TOK_QUOTE: u8 = 5;
const TOK_APOSTROPHE: u8 = 6;

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

    t[b'#' as usize] = TOK_HASH;
    t[b'"' as usize] = TOK_QUOTE;
    t[b'\'' as usize] = TOK_APOSTROPHE;
    t
};

pub const TOKEN_LUT_LO: [i8; 32] = make_token_lut_lo(&TOKEN_KIND);

#[inline(always)]
const fn len_matches(n: usize, idx: usize) -> bool {
    n == KEYWORD_LENS[idx] as usize
}

#[cfg(test)]
#[inline(always)]
pub fn is_julia_extension(name: &[u8]) -> bool {
    super::detect_language(name, super::LANG_ALL_MASK) == Some(super::LangId::Julia)
}

pub struct JuliaLexer;

impl LangLexer<KEYWORD_COUNT> for JuliaLexer {
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
            TOK_HASH => {
                if i + 1 < content_len && content[i + 1] == b'=' {
                    *line_state |= STATE_COMMENT;
                    let end_i = skip_nested_hash_equal_comment(content, i + 2);
                    let nl = count_newlines_avx2(&content[i + 2..end_i]);
                    if nl > 0 {
                        if (*line_state & STATE_CODE) != 0 {
                            acc.code_lines += 1;
                        } else {
                            acc.comment_lines += 1;
                        }
                        acc.comment_lines += (nl - 1) as i64;
                        acc.lines_processed += nl as i64;
                        *line_state = STATE_COMMENT;
                    }
                    end_i
                } else {
                    *line_state |= STATE_COMMENT;
                    let Some(pos) = find_byte_avx2(&content[i + 1..], b'\n') else {
                        return content_len;
                    };
                    let nl_idx = i + 1 + pos;
                    if (*line_state & STATE_CODE) != 0 {
                        acc.code_lines += 1;
                    } else if (*line_state & STATE_COMMENT) != 0 {
                        acc.comment_lines += 1;
                    } else {
                        acc.blank_lines += 1;
                    }
                    acc.lines_processed += 1;
                    *line_state = STATE_NONE;
                    nl_idx + 1
                }
            }
            TOK_QUOTE => {
                *line_state |= STATE_CODE;
                if i + 2 < content_len && content[i + 1] == b'"' && content[i + 2] == b'"' {
                    let end_i = skip_triple_literal(content, i + 3, b'"');
                    let nl = count_newlines_avx2(&content[i + 3..end_i]);
                    if nl > 0 {
                        acc.code_lines += nl as i64;
                        acc.lines_processed += nl as i64;
                    }
                    end_i
                } else {
                    let end_i = skip_literal(content, i + 1, b'"');
                    let nl = count_newlines_avx2(&content[i + 1..end_i]);
                    if nl > 0 {
                        acc.code_lines += nl as i64;
                        acc.lines_processed += nl as i64;
                    }
                    end_i
                }
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

pub fn parse_into(content: &[u8], acc: &mut ParseResult<KEYWORD_COUNT>) {
    unified_parse_into::<KEYWORD_COUNT, JuliaLexer>(content, acc);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_julia_extensions() {
        assert!(is_julia_extension(b"script.jl"));
        assert!(!is_julia_extension(b"script.py"));
        assert!(!is_julia_extension(b"main.rs"));
    }

    #[test]
    fn test_julia_keyword_table_and_mph() {
        for (i, name) in KEYWORD_NAMES.iter().enumerate() {
            let b = name.as_bytes();
            let h = kw_hash(b[0], b[b.len() / 2], b[b.len() - 1], b.len());
            assert_eq!(KW_TABLE[h], i as u8);
            assert_eq!(JuliaLexer::match_keyword(b), Some(i));
        }
        assert_eq!(JuliaLexer::match_keyword(b"notakeyword"), None);
        assert_eq!(JuliaLexer::match_keyword(b"julia"), None);
    }

    #[test]
    fn test_julia_comments_and_strings() {
        let src = b"# line comment\nmodule Math\n#= block\n   comment =#\nfunction add(a, b)\n    str = \"\"\"multi\nline\"\"\"\n    return a + b\nend\nend\n";
        let mut acc = ParseResult::default();
        parse_into(src, &mut acc);
        assert_eq!(acc.comment_lines, 3);
        assert_eq!(acc.code_lines, 7);
        assert_eq!(acc.blank_lines, 0);
        assert!(acc.total_keywords >= 4);
    }
}
