use super::{
    compute_keyword_lens, compute_kw_max_len, compute_kw_min_len, finish_line_comment,
    handle_quoted_literal, make_token_kind, make_token_lut_lo, match_keyword_with,
    unified_parse_into, LangLexer, ParseResult, KW_EMPTY, STATE_CODE,
};

pub const KEYWORD_COUNT: usize = 17;
const KW_SLOT_MASK: usize = 127;

pub const KEYWORD_NAMES: [&str; KEYWORD_COUNT] = [
    "define",
    "else",
    "endef",
    "endif",
    "export",
    "if",
    "ifdef",
    "ifeq",
    "ifndef",
    "ifneq",
    "include",
    "override",
    "private",
    "sinclude",
    "undefine",
    "unexport",
    "vpath",
];

pub const KEYWORD_LENS: [u8; KEYWORD_COUNT] = compute_keyword_lens(&KEYWORD_NAMES);
pub const KW_MIN_LEN: usize = compute_kw_min_len(&KEYWORD_NAMES);
pub const KW_MAX_LEN: usize = compute_kw_max_len(&KEYWORD_NAMES);

#[inline(always)]
const fn kw_hash(first: u8, second: u8, fourth: u8, last: u8, len: usize) -> usize {
    let val = (first as usize)
        .wrapping_mul(435)
        .wrapping_add((second as usize).wrapping_mul(9196))
        .wrapping_add((fourth as usize).wrapping_mul(3258))
        .wrapping_add((last as usize).wrapping_mul(8929))
        .wrapping_add(len.wrapping_mul(6874));
    (val ^ (val >> 2)) & KW_SLOT_MASK
}

const KW_TABLE: [u8; 128] = {
    let mut table = [KW_EMPTY; 128];
    let mut i = 0;
    while i < KEYWORD_COUNT {
        let name = KEYWORD_NAMES[i].as_bytes();
        let len = KEYWORD_LENS[i] as usize;
        let b3 = if len > 3 { name[3] } else { name[0] };
        let h = kw_hash(name[0], name[1], b3, name[len - 1], len);
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

pub const TOKEN_KIND: [u8; 256] = make_token_kind(
    TOK_IDENT,
    &[
        (b'-', TOK_IDENT),
        (b'#', TOK_HASH),
        (b'"', TOK_QUOTE),
        (b'\'', TOK_APOSTROPHE),
    ],
);

pub const TOKEN_LUT_LO: [i8; 32] = make_token_lut_lo(&TOKEN_KIND);

pub struct MakefileLexer;

impl LangLexer<KEYWORD_COUNT> for MakefileLexer {
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
            |c| { let b3 = if c.len() > 3 { c[3] } else { c[0] }; kw_hash(c[0], c[1], b3, c[c.len() - 1], c.len()) },
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
            TOK_HASH => finish_line_comment(content, i + 1, line_state, acc),
            TOK_QUOTE | TOK_APOSTROPHE => {
                handle_quoted_literal(content, i, content[i], line_state, acc)
            }
            _ => {
                *line_state |= STATE_CODE;
                i + 1
            }
        }
    }
}

pub fn parse_into(content: &[u8], res: &mut ParseResult<KEYWORD_COUNT>) {
    unified_parse_into::<KEYWORD_COUNT, MakefileLexer>(content, res);
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_makefile_keyword_table_and_mph() {
        for (i, &name) in KEYWORD_NAMES.iter().enumerate() {
            let idx = MakefileLexer::match_keyword(name.as_bytes());
            assert_eq!(idx, Some(i), "keyword {} failed match", name);
        }
        assert_eq!(MakefileLexer::match_keyword(b"notakeyword"), None);
        assert_eq!(MakefileLexer::match_keyword(b"x"), None);
    }

    #[test]
    fn test_makefile_comments_and_strings() {
        let makefile = b"# Makefile\nall: main.o\n\tgcc -o app main.o\n\ninclude config.mk\n";
        let mut res = ParseResult::default();
        parse_into(makefile, &mut res);
        assert_eq!(res.lines_processed, 5);
        assert_eq!(res.code_lines, 3);
        assert_eq!(res.comment_lines, 1);
        assert_eq!(res.blank_lines, 1);
        assert_eq!(res.total_keywords, 1);
    }
}
