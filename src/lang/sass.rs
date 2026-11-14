use super::{
    compute_keyword_lens, compute_kw_max_len, compute_kw_min_len, handle_c_slash_comments,
    handle_quoted_literal, make_token_kind, make_token_lut_lo, unified_parse_into, LangLexer,
    ParseResult, STATE_CODE,
};

pub const KEYWORD_COUNT: usize = 17;
const KW_SLOT_MASK: usize = 63;
const KW_EMPTY: u8 = 255;

pub const KEYWORD_NAMES: [&str; KEYWORD_COUNT] = [
    "content",
    "debug",
    "each",
    "else",
    "error",
    "extend",
    "for",
    "forward",
    "function",
    "if",
    "import",
    "include",
    "mixin",
    "return",
    "use",
    "warn",
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
        .wrapping_add(mid as usize)
        .wrapping_add(last as usize)
        .wrapping_add(len.wrapping_mul(5));
    val & KW_SLOT_MASK
}

const KW_TABLE: [u8; 64] = {
    let mut table = [KW_EMPTY; 64];
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
const TOK_QUOTE: u8 = 5;
const TOK_SQUOTE: u8 = 6;

pub const TOKEN_KIND: [u8; 256] = make_token_kind(
    TOK_IDENT,
    &[
        (b'-', TOK_IDENT),
        (b'@', TOK_IDENT),
        (b'/', TOK_SLASH),
        (b'"', TOK_QUOTE),
        (b'\'', TOK_SQUOTE),
    ],
);

pub const TOKEN_LUT_LO: [i8; 32] = make_token_lut_lo(&TOKEN_KIND);

#[inline(always)]
const fn len_matches(n: usize, idx: usize) -> bool {
    n == KEYWORD_LENS[idx] as usize
}

pub struct SassLexer;

impl LangLexer<KEYWORD_COUNT> for SassLexer {
    const TOKEN_KIND: [u8; 256] = TOKEN_KIND;
    const TOKEN_LUT_LO: [i8; 32] = TOKEN_LUT_LO;
    const HAS_DOLLAR_IDENT: bool = true;
    const SUPPORTS_TEMPLATES: bool = false;

    #[inline(always)]
    fn match_keyword(cand: &[u8]) -> Option<usize> {
        let cand = if !cand.is_empty() && cand[0] == b'@' {
            &cand[1..]
        } else {
            cand
        };
        let n = cand.len();
        if n.wrapping_sub(KW_MIN_LEN) <= KW_MAX_LEN - KW_MIN_LEN {
            let h = kw_hash(cand[0], cand[n / 2], cand[n - 1], n);
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
        match kind {
            TOK_SLASH => handle_c_slash_comments(content, i, line_state, acc),
            TOK_QUOTE => handle_quoted_literal(content, i, b'"', line_state, acc),
            TOK_SQUOTE => handle_quoted_literal(content, i, b'\'', line_state, acc),
            _ => {
                *line_state |= STATE_CODE;
                i + 1
            }
        }
    }
}

pub fn parse_into(content: &[u8], res: &mut ParseResult<KEYWORD_COUNT>) {
    unified_parse_into::<KEYWORD_COUNT, SassLexer>(content, res);
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_sass_keyword_table_and_mph() {
        for (i, &name) in KEYWORD_NAMES.iter().enumerate() {
            let idx = SassLexer::match_keyword(name.as_bytes());
            assert_eq!(idx, Some(i), "keyword {} failed match", name);
            let mut at_name = Vec::from(b"@");
            at_name.extend_from_slice(name.as_bytes());
            assert_eq!(SassLexer::match_keyword(&at_name), Some(i));
        }
        assert_eq!(SassLexer::match_keyword(b"notakeyword"), None);
        assert_eq!(SassLexer::match_keyword(b"@notakeyword"), None);
    }

    #[test]
    fn test_sass_comments_and_strings() {
        let code = b"// Sass stylesheet\n@mixin reset {\n  margin: 0;\n  padding: 0;\n}\n\n/* multi\n   line */\nbody {\n  @include reset;\n  content: \"test\";\n}\n";
        let mut res = ParseResult::default();
        parse_into(code, &mut res);
        assert_eq!(res.lines_processed, 12);
        assert_eq!(res.comment_lines, 3);
        assert_eq!(res.code_lines, 8);
        assert_eq!(res.blank_lines, 1);
        assert!(res.total_keywords >= 2);
    }
}
