use super::{
    compute_keyword_lens, compute_kw_max_len, compute_kw_min_len, finish_line_comment,
    handle_c_block_comment, handle_html_comment, handle_quoted_literal, kw_hash_b01ml_add,
    make_kw_table_b01ml_add, make_token_kind, make_token_lut_lo, match_keyword_with,
    unified_parse_into, LangLexer, ParseResult, STATE_CODE,
};

pub const KEYWORD_COUNT: usize = 16;
const KW_SLOT_MASK: usize = 31;

pub const KEYWORD_NAMES: [&str; KEYWORD_COUNT] = [
    "Astro",
    "client",
    "defineMiddleware",
    "getCollection",
    "getEntry",
    "getEntryBySlug",
    "getStaticPaths",
    "inferEntrySchema",
    "isDbError",
    "props",
    "redirect",
    "render",
    "request",
    "response",
    "slots",
    "url",
];

pub const KEYWORD_LENS: [u8; KEYWORD_COUNT] = compute_keyword_lens(&KEYWORD_NAMES);
pub const KW_MIN_LEN: usize = compute_kw_min_len(&KEYWORD_NAMES);
pub const KW_MAX_LEN: usize = compute_kw_max_len(&KEYWORD_NAMES);

const TOK_IDENT: u8 = 3;
const TOK_SLASH: u8 = 4;
const TOK_LT: u8 = 5;
const TOK_QUOTE: u8 = 6;
const TOK_APOSTROPHE: u8 = 7;
const TOK_BACKTICK: u8 = 8;

#[inline(always)]
const fn kw_hash(cand: &[u8], _len: usize) -> usize {
    kw_hash_b01ml_add(cand, [4584, 8211, 9460, 933, 7746], 2, KW_SLOT_MASK)
}

const KW_TABLE: [u8; 32] = make_kw_table_b01ml_add(&KEYWORD_NAMES, [4584, 8211, 9460, 933, 7746], 2);

pub const TOKEN_KIND: [u8; 256] = make_token_kind(
    TOK_IDENT,
    &[
        (b'/', TOK_SLASH),
        (b'<', TOK_LT),
        (b'"', TOK_QUOTE),
        (b'\'', TOK_APOSTROPHE),
        (b'`', TOK_BACKTICK),
    ],
);

pub const TOKEN_LUT_LO: [i8; 32] = make_token_lut_lo(&TOKEN_KIND);

pub struct AstroLexer;

impl LangLexer<KEYWORD_COUNT> for AstroLexer {
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
        let content_len = content.len();
        match kind {
            TOK_SLASH => {
                if i + 1 < content_len {
                    let next = content[i + 1];
                    if next == b'/' {
                        return finish_line_comment(content, i + 2, line_state, acc);
                    } else if next == b'*' {
                        return handle_c_block_comment(content, i + 2, line_state, acc);
                    }
                }
                *line_state |= STATE_CODE;
                i + 1
            }
            TOK_LT => {
                if i + 3 < content_len
                    && content[i + 1] == b'!'
                    && content[i + 2] == b'-'
                    && content[i + 3] == b'-'
                {
                    return handle_html_comment(content, i + 4, line_state, acc);
                }
                *line_state |= STATE_CODE;
                i + 1
            }
            TOK_QUOTE => handle_quoted_literal(content, i, b'"', line_state, acc),
            TOK_APOSTROPHE => handle_quoted_literal(content, i, b'\'', line_state, acc),
            TOK_BACKTICK => handle_quoted_literal(content, i, b'`', line_state, acc),
            _ => {
                *line_state |= STATE_CODE;
                i + 1
            }
        }
    }
}

#[inline(always)]
pub fn parse_into(content: &[u8], acc: &mut ParseResult<KEYWORD_COUNT>) {
    unified_parse_into::<KEYWORD_COUNT, AstroLexer>(content, acc);
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
            let idx = AstroLexer::match_keyword(kw.as_bytes()).unwrap();
            assert_eq!(acc.keyword_counts[idx], 1);
        }
    }

    #[test]
    fn test_comments() {
        let text = b"<!-- html comment -->\n// line comment\n/* block comment */\ncode\n";
        let mut acc = ParseResult::default();
        parse_into(text, &mut acc);
        assert_eq!(acc.lines_processed, 4);
        assert_eq!(acc.comment_lines, 3);
        assert_eq!(acc.code_lines, 1);
    }
}
