use super::{
    compute_keyword_lens, compute_kw_max_len, compute_kw_min_len, handle_html_comment,
    handle_quoted_literal, make_token_kind, make_token_lut_lo, unified_parse_into, LangLexer,
    ParseResult, STATE_CODE,
};

pub const KEYWORD_COUNT: usize = 72;
const KW_SLOT_MASK: usize = 255;
const KW_EMPTY: u8 = 255;

pub const KEYWORD_NAMES: [&str; KEYWORD_COUNT] = [
    "abbr", "address", "area", "article", "aside", "audio", "base",
    "bdi", "bdo", "blockquote", "body", "button", "canvas", "caption",
    "cite", "code", "colgroup", "data", "datalist", "details",
    "dfn", "dialog", "embed", "fieldset",
    "figcaption", "figure", "footer", "form", "head", "header", "hgroup",
    "html", "iframe", "input", "label", "legend", "link", "main",
    "mark", "menu", "meta", "meter", "noscript", "object",
    "optgroup", "option", "output", "picture", "progress",
    "ruby", "samp", "script", "search", "section", "select",
    "slot", "small", "source", "span", "strong", "style", "summary",
    "table", "tbody", "template", "textarea", "tfoot",
    "thead", "time", "title", "track", "video",
];

pub const KEYWORD_LENS: [u8; KEYWORD_COUNT] = compute_keyword_lens(&KEYWORD_NAMES);
pub const KW_MIN_LEN: usize = compute_kw_min_len(&KEYWORD_NAMES);
pub const KW_MAX_LEN: usize = compute_kw_max_len(&KEYWORD_NAMES);

const _: () = assert!(KEYWORD_NAMES.len() == KEYWORD_COUNT);
const _: () = assert!(KEYWORD_LENS.len() == KEYWORD_COUNT);
const _: () = assert!(KW_MIN_LEN >= 2 && KW_MAX_LEN <= 16);

#[inline(always)]
const fn kw_hash(first: u8, mid: u8, last: u8, len: usize) -> usize {
    let val = (first as usize).wrapping_mul(2)
        ^ (mid as usize).wrapping_mul(135)
        ^ (last as usize).wrapping_mul(234)
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
const TOK_LT: u8 = 4;
const TOK_QUOTE: u8 = 5;
const TOK_SQUOTE: u8 = 6;

pub const TOKEN_KIND: [u8; 256] = {
    let mut t = make_token_kind(TOK_IDENT, &[(b'-', TOK_IDENT), (b'<', TOK_LT), (b'"', TOK_QUOTE), (b'\'', TOK_SQUOTE)]);
    let mut c = b'0';
    while c <= b'9' {
        t[c as usize] = TOK_IDENT;
        c += 1;
    }
    t
};

pub const TOKEN_LUT_LO: [i8; 32] = make_token_lut_lo(&TOKEN_KIND);

#[inline(always)]
const fn len_matches(n: usize, idx: usize) -> bool {
    n == KEYWORD_LENS[idx] as usize
}

pub struct HtmlLexer;

impl LangLexer<KEYWORD_COUNT> for HtmlLexer {
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
            TOK_LT => {
                if i + 3 < content_len
                    && content[i + 1] == b'!'
                    && content[i + 2] == b'-'
                    && content[i + 3] == b'-'
                {
                    handle_html_comment(content, i + 4, line_state, acc)
                } else {
                    *line_state |= STATE_CODE;
                    i + 1
                }
            }
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
    unified_parse_into::<KEYWORD_COUNT, HtmlLexer>(content, res);
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_html_keyword_table_and_mph() {
        for (i, &name) in KEYWORD_NAMES.iter().enumerate() {
            let idx = HtmlLexer::match_keyword(name.as_bytes());
            assert_eq!(idx, Some(i), "keyword {} failed match", name);
            let upper = name.to_ascii_uppercase();
            let idx_upper = HtmlLexer::match_keyword(upper.as_bytes());
            assert_eq!(idx_upper, Some(i), "keyword {} upper failed match", upper);
        }
        assert_eq!(HtmlLexer::match_keyword(b"notakeyword"), None);
        assert_eq!(HtmlLexer::match_keyword(b"xyz"), None);
    }

    #[test]
    fn test_html_comments_and_strings() {
        let code = b"<!-- HTML page -->\n                     <!DOCTYPE html>\n                     <html>\n                       <body>\n                         <div class=\"container\">\n                           <!-- multi\n                                line -->\n                           <p>Hello</p>\n                         </div>\n                       </body>\n                     </html>\n";
        let mut res = ParseResult::default();
        parse_into(code, &mut res);
        assert_eq!(res.lines_processed, 11);
        assert_eq!(res.comment_lines, 3);
        assert_eq!(res.code_lines, 8);
        assert_eq!(res.blank_lines, 0);
        assert!(res.total_keywords >= 4);
    }
}
