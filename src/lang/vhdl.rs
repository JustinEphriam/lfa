use super::{
    compute_keyword_lens, compute_kw_max_len, compute_kw_min_len, finish_line_comment,
    handle_quoted_literal, kw_hash_fml_xor, make_kw_table_fml_xor, make_token_kind,
    make_token_lut_lo, match_keyword_caseless_with, unified_parse_into, LangLexer, ParseResult,
    STATE_CODE,
};

pub const KEYWORD_COUNT: usize = 97;
const KW_SLOT_MASK: usize = 511;

pub const KEYWORD_NAMES: [&str; KEYWORD_COUNT] = [
    "abs", "access", "after", "alias", "all", "and", "architecture",
    "array", "assert", "attribute", "begin", "block", "body", "buffer",
    "bus", "case", "component", "configuration", "constant", "disconnect",
    "downto", "else", "elsif", "end", "entity", "exit", "file", "for",
    "function", "generate", "generic", "group", "guarded", "if", "impure",
    "in", "inertial", "inout", "is", "label", "library", "linkage",
    "literal", "loop", "map", "mod", "nand", "new", "next", "nor", "not",
    "null", "of", "on", "open", "or", "others", "out", "package", "port",
    "postponed", "procedure", "process", "pure", "range", "record",
    "register", "reject", "rem", "report", "return", "rol", "ror", "select",
    "severity", "shared", "signal", "sla", "sll", "sra", "srl", "subtype",
    "then", "to", "transport", "type", "unaffected", "units", "until",
    "use", "variable", "wait", "when", "while", "with", "xnor", "xor",
];

pub const KEYWORD_LENS: [u8; KEYWORD_COUNT] = compute_keyword_lens(&KEYWORD_NAMES);
pub const KW_MIN_LEN: usize = compute_kw_min_len(&KEYWORD_NAMES);
pub const KW_MAX_LEN: usize = compute_kw_max_len(&KEYWORD_NAMES);

#[inline(always)]
const fn kw_hash(first: u8, mid: u8, last: u8, len: usize) -> usize {
    kw_hash_fml_xor(first, mid, last, len, [1, 17, 341, 17], 0, KW_SLOT_MASK)
}

const KW_TABLE: [u8; 512] = make_kw_table_fml_xor(&KEYWORD_NAMES, [1, 17, 341, 17], 0);

const TOK_IDENT: u8 = 3;
const TOK_MINUS: u8 = 4;
const TOK_QUOTE: u8 = 5;

pub const TOKEN_KIND: [u8; 256] = make_token_kind(
    TOK_IDENT,
    &[
        (b'-', TOK_MINUS),
        (b'"', TOK_QUOTE),
    ],
);

pub const TOKEN_LUT_LO: [i8; 32] = make_token_lut_lo(&TOKEN_KIND);

pub struct VhdlLexer;

impl LangLexer<KEYWORD_COUNT> for VhdlLexer {
    const TOKEN_KIND: [u8; 256] = TOKEN_KIND;
    const TOKEN_LUT_LO: [i8; 32] = TOKEN_LUT_LO;
    const HAS_DOLLAR_IDENT: bool = false;
    const SUPPORTS_TEMPLATES: bool = false;

    #[inline(always)]
    fn match_keyword(cand: &[u8]) -> Option<usize> {
        match_keyword_caseless_with(
            cand,
            KW_MIN_LEN,
            KW_MAX_LEN,
            &KW_TABLE,
            &KEYWORD_LENS,
            &KEYWORD_NAMES,
            |c| kw_hash(c[0].to_ascii_lowercase(), c[c.len() / 2].to_ascii_lowercase(), c[c.len() - 1].to_ascii_lowercase(), c.len()),
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
            TOK_MINUS => {
                if i + 1 < content_len && content[i + 1] == b'-' {
                    finish_line_comment(content, i + 2, line_state, acc)
                } else {
                    *line_state |= STATE_CODE;
                    i + 1
                }
            }
            TOK_QUOTE => handle_quoted_literal(content, i, b'"', line_state, acc),
            _ => {
                *line_state |= STATE_CODE;
                i + 1
            }
        }
    }
}

pub fn parse_into(content: &[u8], res: &mut ParseResult<KEYWORD_COUNT>) {
    unified_parse_into::<KEYWORD_COUNT, VhdlLexer>(content, res);
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_vhdl_keyword_table_and_mph() {
        for (i, &name) in KEYWORD_NAMES.iter().enumerate() {
            let idx = VhdlLexer::match_keyword(name.as_bytes());
            assert_eq!(idx, Some(i), "keyword {} failed match", name);
            let upper = name.to_ascii_uppercase();
            let idx_upper = VhdlLexer::match_keyword(upper.as_bytes());
            assert_eq!(idx_upper, Some(i), "keyword {} upper failed match", upper);
        }
        assert_eq!(VhdlLexer::match_keyword(b"notakeyword"), None);
        assert_eq!(VhdlLexer::match_keyword(b"x"), None);
    }

    #[test]
    fn test_vhdl_comments_and_strings() {
        let code = b"-- VHDL design
                     library IEEE;
                     use IEEE.STD_LOGIC_1164.ALL;
                     entity Counter is
                         port (clk: in STD_LOGIC); -- clock input
                     end Counter;
";
        let mut res = ParseResult::default();
        parse_into(code, &mut res);
        assert_eq!(res.lines_processed, 6);
        assert_eq!(res.comment_lines, 1);
        assert_eq!(res.code_lines, 5);
        assert_eq!(res.blank_lines, 0);
        assert!(res.total_keywords >= 4);
    }
}
