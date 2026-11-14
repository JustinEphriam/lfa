use super::{
    compute_keyword_lens, compute_kw_max_len, compute_kw_min_len, handle_c_slash_comments,
    handle_quoted_literal, kw_hash_fml_add, make_kw_table_fml_add, make_token_kind,
    make_token_lut_lo, match_keyword_with, unified_parse_into, LangLexer, ParseResult, STATE_CODE,
};

pub const KEYWORD_COUNT: usize = 56;
const KW_SLOT_MASK: usize = 255;

pub const KEYWORD_NAMES: [&str; KEYWORD_COUNT] = [
    "bool",
    "break",
    "cbuffer",
    "centroid",
    "column_major",
    "compile",
    "const",
    "continue",
    "default",
    "discard",
    "do",
    "double",
    "dword",
    "else",
    "export",
    "extern",
    "false",
    "float",
    "for",
    "groupshared",
    "half",
    "if",
    "in",
    "inline",
    "inout",
    "int",
    "linear",
    "matrix",
    "namespace",
    "nointerpolation",
    "noperspective",
    "null",
    "out",
    "packoffset",
    "precise",
    "register",
    "return",
    "row_major",
    "sample",
    "sampler",
    "shared",
    "snorm",
    "static",
    "struct",
    "switch",
    "tbuffer",
    "technique",
    "true",
    "typedef",
    "uint",
    "uniform",
    "unorm",
    "vector",
    "void",
    "volatile",
    "while",
];

pub const KEYWORD_LENS: [u8; KEYWORD_COUNT] = compute_keyword_lens(&KEYWORD_NAMES);
pub const KW_MIN_LEN: usize = compute_kw_min_len(&KEYWORD_NAMES);
pub const KW_MAX_LEN: usize = compute_kw_max_len(&KEYWORD_NAMES);

#[inline(always)]
const fn kw_hash(first: u8, mid: u8, last: u8, len: usize) -> usize {
    kw_hash_fml_add(first, mid, last, len, [7455, 4674, 6931, 9106], 4, KW_SLOT_MASK)
}

const KW_TABLE: [u8; 256] = make_kw_table_fml_add(&KEYWORD_NAMES, [7455, 4674, 6931, 9106], 4);

const TOK_IDENT: u8 = 3;
const TOK_SLASH: u8 = 4;
const TOK_QUOTE: u8 = 5;

pub const TOKEN_KIND: [u8; 256] = make_token_kind(
    TOK_IDENT,
    &[
        (b'/', TOK_SLASH),
        (b'"', TOK_QUOTE),
    ],
);

pub const TOKEN_LUT_LO: [i8; 32] = make_token_lut_lo(&TOKEN_KIND);

pub struct HlslLexer;

impl LangLexer<KEYWORD_COUNT> for HlslLexer {
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
            TOK_SLASH => handle_c_slash_comments(content, i, line_state, acc),
            TOK_QUOTE => handle_quoted_literal(content, i, b'"', line_state, acc),
            _ => {
                *line_state |= STATE_CODE;
                i + 1
            }
        }
    }
}

pub fn parse_into(content: &[u8], res: &mut ParseResult<KEYWORD_COUNT>) {
    unified_parse_into::<KEYWORD_COUNT, HlslLexer>(content, res);
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_hlsl_keyword_table_and_mph() {
        for (i, &name) in KEYWORD_NAMES.iter().enumerate() {
            let idx = HlslLexer::match_keyword(name.as_bytes());
            assert_eq!(idx, Some(i), "keyword {} failed match", name);
        }
        assert_eq!(HlslLexer::match_keyword(b"notakeyword"), None);
        assert_eq!(HlslLexer::match_keyword(b"x"), None);
    }

    #[test]
    fn test_hlsl_comments_and_strings() {
        let hlsl = b"// Pixel shader\nstruct VS_OUTPUT {\n    float4 Pos : SV_POSITION;\n};\n/* block\ncomment */\nfloat4 main(VS_OUTPUT input) : SV_TARGET {\n    return float4(1.0, 0.0, 0.0, 1.0);\n}\n";
        let mut res = ParseResult::default();
        parse_into(hlsl, &mut res);
        assert_eq!(res.lines_processed, 9);
        assert_eq!(res.code_lines, 6);
        assert_eq!(res.comment_lines, 3);
        assert_eq!(res.blank_lines, 0);
        assert!(res.total_keywords >= 2);
    }
}
