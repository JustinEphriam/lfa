use super::{
    compute_keyword_lens, compute_kw_max_len, compute_kw_min_len, handle_quoted_literal,
    handle_spliced_c_slash_comments, kw_match_2_to_16, make_token_kind, make_token_lut_lo,
    unified_parse_into, LangLexer, ParseResult, STATE_CODE,
};

pub const KEYWORD_COUNT: usize = 81;
const KW_SLOT_MASK: usize = 255;
const KW_EMPTY: u8 = 255;

pub const KEYWORD_NAMES: [&str; KEYWORD_COUNT] = [
    "__activemask",
    "__ballot_sync",
    "__constant__",
    "__device__",
    "__forceinline__",
    "__global__",
    "__host__",
    "__inline__",
    "__managed__",
    "__noinline__",
    "__restrict__",
    "__shared__",
    "__shfl_down_sync",
    "__shfl_sync",
    "__shfl_up_sync",
    "__shfl_xor_sync",
    "__syncthreads",
    "__syncthreads_or",
    "__syncwarp",
    "__threadfence",
    "auto",
    "blockDim",
    "blockIdx",
    "break",
    "case",
    "catch",
    "char",
    "class",
    "const",
    "continue",
    "default",
    "delete",
    "do",
    "double",
    "else",
    "enum",
    "explicit",
    "export",
    "extern",
    "false",
    "float",
    "for",
    "friend",
    "goto",
    "gridDim",
    "if",
    "inline",
    "int",
    "long",
    "mutable",
    "namespace",
    "new",
    "operator",
    "private",
    "protected",
    "public",
    "register",
    "return",
    "short",
    "signed",
    "sizeof",
    "static",
    "struct",
    "switch",
    "template",
    "this",
    "threadIdx",
    "throw",
    "true",
    "try",
    "typedef",
    "typeid",
    "typename",
    "union",
    "unsigned",
    "using",
    "virtual",
    "void",
    "volatile",
    "warpSize",
    "while",
];

pub const KEYWORD_LENS: [u8; KEYWORD_COUNT] = compute_keyword_lens(&KEYWORD_NAMES);
pub const KW_MIN_LEN: usize = compute_kw_min_len(&KEYWORD_NAMES);
pub const KW_MAX_LEN: usize = compute_kw_max_len(&KEYWORD_NAMES);

const _: () = assert!(KEYWORD_NAMES.len() == KEYWORD_COUNT);
const _: () = assert!(KEYWORD_LENS.len() == KEYWORD_COUNT);
const _: () = assert!(KW_MIN_LEN >= 2 && KW_MAX_LEN <= 16);

#[inline(always)]
const fn kw_hash(b0: u8, b1: u8, b2: u8, mid: u8, last: u8, len: usize) -> usize {
    let val = (b0 as usize)
        .wrapping_mul(3516)
        .wrapping_add((b1 as usize).wrapping_mul(3256))
        .wrapping_add((b2 as usize).wrapping_mul(5417))
        .wrapping_add((mid as usize).wrapping_mul(5429))
        .wrapping_add((last as usize).wrapping_mul(566))
        .wrapping_add(len.wrapping_mul(5310));
    (val ^ (val >> 5)) & KW_SLOT_MASK
}

const KW_TABLE: [u8; 256] = {
    let mut table = [KW_EMPTY; 256];
    let mut i = 0;
    while i < KEYWORD_COUNT {
        let name = KEYWORD_NAMES[i].as_bytes();
        let len = KEYWORD_LENS[i] as usize;
        assert!(name.len() == len);
        assert!(len >= KW_MIN_LEN && len <= KW_MAX_LEN);
        let b2 = if len > 2 { name[2] } else { name[0] };
        let h = kw_hash(name[0], name[1], b2, name[len / 2], name[len - 1], len);
        assert!(table[h] == KW_EMPTY);
        table[h] = i as u8;
        i += 1;
    }
    table
};

const TOK_IDENT: u8 = 3;
const TOK_SLASH: u8 = 4;
const TOK_QUOTE: u8 = 5;
const TOK_APOSTROPHE: u8 = 6;

pub const TOKEN_KIND: [u8; 256] = make_token_kind(
    TOK_IDENT,
    &[
        (b'/', TOK_SLASH),
        (b'"', TOK_QUOTE),
        (b'\'', TOK_APOSTROPHE),
    ],
);

pub const TOKEN_LUT_LO: [i8; 32] = make_token_lut_lo(&TOKEN_KIND);

#[inline(always)]
const fn len_matches(n: usize, idx: usize) -> bool {
    n == KEYWORD_LENS[idx] as usize
}

pub struct CudaLexer;

impl LangLexer<KEYWORD_COUNT> for CudaLexer {
    const TOKEN_KIND: [u8; 256] = TOKEN_KIND;
    const TOKEN_LUT_LO: [i8; 32] = TOKEN_LUT_LO;
    const HAS_DOLLAR_IDENT: bool = false;
    const SUPPORTS_TEMPLATES: bool = false;

    #[inline(always)]
    fn match_keyword(cand: &[u8]) -> Option<usize> {
        let n = cand.len();
        if n.wrapping_sub(KW_MIN_LEN) <= KW_MAX_LEN - KW_MIN_LEN {
            let b2 = if n > 2 { cand[2] } else { cand[0] };
            let h = kw_hash(cand[0], cand[1], b2, cand[n / 2], cand[n - 1], n);
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
        match kind {
            TOK_SLASH => handle_spliced_c_slash_comments(content, i, line_state, acc),
            TOK_QUOTE => handle_quoted_literal(content, i, b'"', line_state, acc),
            TOK_APOSTROPHE => handle_quoted_literal(content, i, b'\'', line_state, acc),
            _ => {
                *line_state |= STATE_CODE;
                i + 1
            }
        }
    }
}

pub fn parse_into(content: &[u8], res: &mut ParseResult<KEYWORD_COUNT>) {
    unified_parse_into::<KEYWORD_COUNT, CudaLexer>(content, res);
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_cuda_keyword_table_and_mph() {
        for (i, &name) in KEYWORD_NAMES.iter().enumerate() {
            let idx = CudaLexer::match_keyword(name.as_bytes());
            assert_eq!(idx, Some(i), "keyword {} failed match", name);
        }
        assert_eq!(CudaLexer::match_keyword(b"notakeyword"), None);
        assert_eq!(CudaLexer::match_keyword(b"x"), None);
    }

    #[test]
    fn test_cuda_comments_and_strings() {
        let code = b"// CUDA vector add kernel\n\
                     __global__ void vecAdd(float* a, float* b, float* c) {\n\
                         int i = blockIdx.x * blockDim.x + threadIdx.x;\n\
                         /* synchronize threads */\n\
                         __syncthreads();\n\
                     }\n";
        let mut res = ParseResult::default();
        parse_into(code, &mut res);
        assert_eq!(res.lines_processed, 6);
        assert_eq!(res.comment_lines, 2);
        assert_eq!(res.code_lines, 4);
        assert_eq!(res.blank_lines, 0);
        assert!(res.total_keywords >= 5);
    }
}
