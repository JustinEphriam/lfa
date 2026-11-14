use super::{
    compute_keyword_lens, compute_kw_max_len, compute_kw_min_len, handle_quoted_literal,
    handle_spliced_c_slash_comments, kw_hash_fml_add, make_kw_table_fml_add, make_token_kind,
    make_token_lut_lo, match_keyword_with, unified_parse_into, LangLexer, ParseResult, STATE_CODE,
};

pub const KEYWORD_COUNT: usize = 53;
const KW_SLOT_MASK: usize = 255;

pub const KEYWORD_NAMES: [&str; KEYWORD_COUNT] = [
    "always",
    "and",
    "assign",
    "begin",
    "buf",
    "case",
    "casex",
    "casez",
    "deassign",
    "default",
    "defparam",
    "disable",
    "edge",
    "else",
    "end",
    "endcase",
    "endfunction",
    "endmodule",
    "endprimitive",
    "endspecify",
    "endtable",
    "endtask",
    "for",
    "force",
    "forever",
    "fork",
    "function",
    "if",
    "initial",
    "inout",
    "input",
    "join",
    "module",
    "nand",
    "negedge",
    "nor",
    "not",
    "or",
    "output",
    "parameter",
    "posedge",
    "primitive",
    "reg",
    "release",
    "repeat",
    "specify",
    "table",
    "task",
    "wait",
    "while",
    "wire",
    "xnor",
    "xor",
];

pub const KEYWORD_LENS: [u8; KEYWORD_COUNT] = compute_keyword_lens(&KEYWORD_NAMES);
pub const KW_MIN_LEN: usize = compute_kw_min_len(&KEYWORD_NAMES);
pub const KW_MAX_LEN: usize = compute_kw_max_len(&KEYWORD_NAMES);

#[inline(always)]
const fn kw_hash(first: u8, mid: u8, last: u8, len: usize) -> usize {
    kw_hash_fml_add(first, mid, last, len, [4049, 1239, 841, 7686], 4, KW_SLOT_MASK)
}

const KW_TABLE: [u8; 256] = make_kw_table_fml_add(&KEYWORD_NAMES, [4049, 1239, 841, 7686], 4);

const TOK_IDENT: u8 = 3;
const TOK_SLASH: u8 = 4;
const TOK_QUOTE: u8 = 5;

pub const TOKEN_KIND: [u8; 256] = make_token_kind(
    TOK_IDENT,
    &[
        (b'$', TOK_IDENT),
        (b'/', TOK_SLASH),
        (b'"', TOK_QUOTE),
    ],
);

pub const TOKEN_LUT_LO: [i8; 32] = make_token_lut_lo(&TOKEN_KIND);

pub struct VerilogLexer;

impl LangLexer<KEYWORD_COUNT> for VerilogLexer {
    const TOKEN_KIND: [u8; 256] = TOKEN_KIND;
    const TOKEN_LUT_LO: [i8; 32] = TOKEN_LUT_LO;
    const HAS_DOLLAR_IDENT: bool = true;
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
            |c| kw_hash(c[0], c[c.len() / 2], c[c.len() - 1], c.len()),
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
            TOK_SLASH => handle_spliced_c_slash_comments(content, i, line_state, acc),
            TOK_QUOTE => handle_quoted_literal(content, i, b'"', line_state, acc),
            _ => {
                *line_state |= STATE_CODE;
                i + 1
            }
        }
    }
}

pub fn parse_into(content: &[u8], res: &mut ParseResult<KEYWORD_COUNT>) {
    unified_parse_into::<KEYWORD_COUNT, VerilogLexer>(content, res);
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_verilog_keyword_table_and_mph() {
        for (i, &name) in KEYWORD_NAMES.iter().enumerate() {
            let idx = VerilogLexer::match_keyword(name.as_bytes());
            assert_eq!(idx, Some(i), "keyword {} failed match", name);
        }
        assert_eq!(VerilogLexer::match_keyword(b"notakeyword"), None);
        assert_eq!(VerilogLexer::match_keyword(b"x"), None);
    }

    #[test]
    fn test_verilog_comments_and_strings() {
        let code = b"// Verilog module\n\
                     module counter(input clk, output reg [3:0] q);\n\
                         /* multiline\n\
                            comment */\n\
                         always @(posedge clk) begin\n\
                             q <= q + 1;\n\
                         end\n\
                     endmodule\n";
        let mut res = ParseResult::default();
        parse_into(code, &mut res);
        assert_eq!(res.lines_processed, 8);
        assert_eq!(res.comment_lines, 3);
        assert_eq!(res.code_lines, 5);
        assert_eq!(res.blank_lines, 0);
        assert!(res.total_keywords >= 5);
    }
}
