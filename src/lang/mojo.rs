use super::{
    compute_keyword_lens, compute_kw_max_len, compute_kw_min_len, count_newlines_avx2,
    find_byte_avx2, kw_match_2_to_16, make_token_lut_lo, skip_literal, skip_triple_literal,
    unified_parse_into, LangLexer, ParseResult, STATE_CODE, STATE_COMMENT, STATE_NONE, TOK_CODE,
    TOK_NL, TOK_WS,
};

pub const KEYWORD_COUNT: usize = 44;
const KW_SLOT_MASK: usize = 255;
const KW_EMPTY: u8 = 255;

pub const KEYWORD_NAMES: [&str; KEYWORD_COUNT] = [
    "alias",
    "and",
    "as",
    "assert",
    "async",
    "await",
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "def",
    "del",
    "deref",
    "elif",
    "else",
    "except",
    "finally",
    "fn",
    "for",
    "from",
    "global",
    "if",
    "import",
    "in",
    "is",
    "lambda",
    "let",
    "match",
    "nonlocal",
    "not",
    "or",
    "pass",
    "raise",
    "ref",
    "return",
    "struct",
    "trait",
    "try",
    "var",
    "while",
    "with",
    "yield",
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
        .wrapping_mul(4504)
        .wrapping_add((mid as usize).wrapping_mul(2900))
        .wrapping_add((last as usize).wrapping_mul(9216))
        .wrapping_add(len.wrapping_mul(7052));
    (val ^ (val >> 4)) & KW_SLOT_MASK
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

pub struct MojoLexer;

impl LangLexer<KEYWORD_COUNT> for MojoLexer {
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
            TOK_QUOTE => {
                *line_state |= STATE_CODE;
                let end_i = if i + 2 < content_len && content[i + 1] == b'"' && content[i + 2] == b'"' {
                    skip_triple_literal(content, i + 3, b'"')
                } else {
                    skip_literal(content, i + 1, b'"')
                };
                let nl = count_newlines_avx2(&content[i + 1..end_i]);
                if nl > 0 {
                    acc.code_lines += nl as i64;
                    acc.lines_processed += nl as i64;
                }
                end_i
            }
            TOK_APOSTROPHE => {
                *line_state |= STATE_CODE;
                let end_i = if i + 2 < content_len && content[i + 1] == b'\'' && content[i + 2] == b'\'' {
                    skip_triple_literal(content, i + 3, b'\'')
                } else {
                    skip_literal(content, i + 1, b'\'')
                };
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
    unified_parse_into::<KEYWORD_COUNT, MojoLexer>(content, res);
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_mojo_keyword_table_and_mph() {
        for (i, &name) in KEYWORD_NAMES.iter().enumerate() {
            let idx = MojoLexer::match_keyword(name.as_bytes());
            assert_eq!(idx, Some(i), "keyword {} failed match", name);
        }
        assert_eq!(MojoLexer::match_keyword(b"notakeyword"), None);
        assert_eq!(MojoLexer::match_keyword(b"x"), None);
    }

    #[test]
    fn test_mojo_comments_and_strings() {
        let code = b"# Mojo script\n\
                     fn main():\n\
                         var x: Int = 42\n\
                         print(\"hello mojo\")\n\
                         let multi = \"\"\"\n\
                         multi line\n\
                         string\n\
                         \"\"\"\n";
        let mut res = ParseResult::default();
        parse_into(code, &mut res);
        assert_eq!(res.lines_processed, 8);
        assert_eq!(res.comment_lines, 1);
        assert_eq!(res.code_lines, 7);
        assert_eq!(res.blank_lines, 0);
        assert!(res.total_keywords >= 3);
    }
}
