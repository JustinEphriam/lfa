use super::{
    compute_keyword_lens, compute_kw_max_len, compute_kw_min_len, finish_line_comment,
    handle_quoted_literal, make_token_kind, make_token_lut_lo,
    match_keyword_caseless_with, unified_parse_into, LangLexer, ParseResult, STATE_CODE,
};

pub const KEYWORD_COUNT: usize = 61;
const KW_SLOT_MASK: usize = 255;
const KW_EMPTY: u8 = 255;

pub const KEYWORD_NAMES: [&str; KEYWORD_COUNT] = [
    "append",
    "at",
    "by",
    "call",
    "case",
    "catch",
    "check",
    "class",
    "clear",
    "commit",
    "constants",
    "continue",
    "data",
    "delete",
    "do",
    "else",
    "elseif",
    "endat",
    "endcase",
    "endclass",
    "enddo",
    "endform",
    "endfunction",
    "endif",
    "endinterface",
    "endloop",
    "endmethod",
    "endtry",
    "endwhile",
    "exit",
    "form",
    "from",
    "function",
    "group",
    "having",
    "if",
    "insert",
    "interface",
    "into",
    "loop",
    "message",
    "method",
    "modify",
    "order",
    "perform",
    "program",
    "raise",
    "read",
    "report",
    "return",
    "rollback",
    "select",
    "sort",
    "tables",
    "try",
    "types",
    "update",
    "when",
    "where",
    "while",
    "write",
];

pub const KEYWORD_LENS: [u8; KEYWORD_COUNT] = compute_keyword_lens(&KEYWORD_NAMES);
pub const KW_MIN_LEN: usize = compute_kw_min_len(&KEYWORD_NAMES);
pub const KW_MAX_LEN: usize = compute_kw_max_len(&KEYWORD_NAMES);

const _: () = assert!(KEYWORD_NAMES.len() == KEYWORD_COUNT);
const _: () = assert!(KEYWORD_LENS.len() == KEYWORD_COUNT);
const _: () = assert!(KW_MIN_LEN >= 2 && KW_MAX_LEN <= 12);

#[inline(always)]
const fn kw_hash(cand: &[u8], len: usize) -> usize {
    let b0 = (cand[0] | 0x20) as usize;
    let bmid = (cand[len / 2] | 0x20) as usize;
    let blast = (cand[len - 1] | 0x20) as usize;
    let b1 = (cand[1] | 0x20) as usize;
    let val = len.wrapping_mul(1404)
        .wrapping_add(b0.wrapping_mul(1576))
        .wrapping_add(b1.wrapping_mul(1248))
        .wrapping_add(bmid.wrapping_mul(6467))
        .wrapping_add(blast.wrapping_mul(6444));
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
        let h = kw_hash(name, len);
        assert!(table[h] == KW_EMPTY);
        table[h] = i as u8;
        i += 1;
    }
    table
};

const TOK_IDENT: u8 = 3;
const TOK_DQUOTE: u8 = 4;
const TOK_SQUOTE: u8 = 5;

pub const TOKEN_KIND: [u8; 256] = make_token_kind(
    TOK_IDENT,
    &[
        (b'"', TOK_DQUOTE),
        (b'\'', TOK_SQUOTE),
    ],
);

pub const TOKEN_LUT_LO: [i8; 32] = make_token_lut_lo(&TOKEN_KIND);

#[cfg(test)]
#[inline(always)]
pub fn is_abap_extension(name: &[u8]) -> bool {
    super::detect_language(name, super::LANG_ALL_MASK) == Some(super::LangId::Abap)
}

pub struct AbapLexer;

impl LangLexer<KEYWORD_COUNT> for AbapLexer {
    const TOKEN_KIND: [u8; 256] = TOKEN_KIND;
    const TOKEN_LUT_LO: [i8; 32] = TOKEN_LUT_LO;
    const HAS_DOLLAR_IDENT: bool = false;
    const SUPPORTS_TEMPLATES: bool = false;
    const HAS_LINE_START_HOOK: bool = true;

    #[inline(always)]
    fn match_keyword(cand: &[u8]) -> Option<usize> {
        match_keyword_caseless_with(
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
    fn handle_line_start(
        content: &[u8],
        i: usize,
        line_state: &mut u8,
        acc: &mut ParseResult<KEYWORD_COUNT>,
    ) -> usize {
        if i < content.len() && content[i] == b'*' {
            return finish_line_comment(content, i + 1, line_state, acc);
        }
        i
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
            TOK_DQUOTE => finish_line_comment(content, i + 1, line_state, acc),
            TOK_SQUOTE => handle_quoted_literal(content, i, b'\'', line_state, acc),
            _ => {
                *line_state |= STATE_CODE;
                i + 1
            }
        }
    }
}

#[inline(always)]
pub fn parse_into(content: &[u8], acc: &mut ParseResult<KEYWORD_COUNT>) {
    unified_parse_into::<KEYWORD_COUNT, AbapLexer>(content, acc);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_abap_keyword_table_and_mph() {
        assert_eq!(KEYWORD_COUNT, 61);
        for i in 0..KEYWORD_COUNT {
            let kw = KEYWORD_NAMES[i];
            let len = kw.len();
            assert_eq!(len, KEYWORD_LENS[i] as usize);
            let bytes = kw.as_bytes();
            let h = kw_hash(bytes, len);
            assert_eq!(KW_TABLE[h], i as u8);

            let mut res = ParseResult::default();
            parse_into(bytes, &mut res);
            assert_eq!(res.total_keywords, 1);
            assert_eq!(res.keyword_counts[i], 1);
            assert_eq!(res.lines_processed, 1);

            let upper = kw.to_ascii_uppercase();
            let mut res_upper = ParseResult::default();
            parse_into(upper.as_bytes(), &mut res_upper);
            assert_eq!(res_upper.total_keywords, 1);
            assert_eq!(res_upper.keyword_counts[i], 1);
        }

        let non_keywords = ["abaps", "programs", "classes", "reports"];
        for s in non_keywords {
            let mut res = ParseResult::default();
            parse_into(s.as_bytes(), &mut res);
            assert_eq!(res.total_keywords, 0);
        }
    }

    #[test]
    fn test_abap_comments_and_strings() {
        let text = concat!(
            "* Full line comment in column 0\n",
            "\" Another full comment line\n",
            "REPORT z_hello_world.\n",
            "\n",
            "DATA: lv_text TYPE string.\n",
            "lv_text = 'Hello, World!'. \" Inline comment\n",
            "WRITE: / lv_text.\n"
        );
        let mut res = ParseResult::default();
        parse_into(text.as_bytes(), &mut res);
        assert_eq!(res.lines_processed, 7);
        assert_eq!(res.comment_lines, 2);
        assert_eq!(res.code_lines, 4);
        assert_eq!(res.blank_lines, 1);
    }

    #[test]
    fn test_abap_extensions() {
        assert!(is_abap_extension(b"report.abap"));
        assert!(!is_abap_extension(b"report.aba"));
    }
}
