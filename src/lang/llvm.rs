use super::{
    compute_keyword_lens, compute_kw_max_len, compute_kw_min_len, finish_line_comment,
    handle_quoted_literal, make_token_kind, make_token_lut_lo, unified_parse_into, LangLexer,
    ParseResult, STATE_CODE,
};

pub const KEYWORD_COUNT: usize = 73;
const KW_SLOT_MASK: usize = 255;
const KW_EMPTY: u8 = 255;

pub const KEYWORD_NAMES: [&str; KEYWORD_COUNT] = [
    "add",
    "alloca",
    "and",
    "ashr",
    "atomicrmw",
    "bitcast",
    "br",
    "call",
    "cmpxchg",
    "constant",
    "datalayout",
    "declare",
    "define",
    "eq",
    "exact",
    "extractelement",
    "extractvalue",
    "fadd",
    "fcmp",
    "fdiv",
    "fence",
    "fmul",
    "fneg",
    "fpext",
    "fptosi",
    "fptoui",
    "fptrunc",
    "freeze",
    "frem",
    "fsub",
    "getelementptr",
    "global",
    "icmp",
    "indirectbr",
    "insertelement",
    "insertvalue",
    "inttoptr",
    "invoke",
    "landingpad",
    "load",
    "lshr",
    "metadata",
    "mul",
    "ne",
    "or",
    "phi",
    "private",
    "ptrtoint",
    "resume",
    "ret",
    "sdiv",
    "select",
    "sext",
    "shl",
    "shufflevector",
    "sitofp",
    "srem",
    "store",
    "sub",
    "switch",
    "target",
    "triple",
    "trunc",
    "type",
    "udiv",
    "uitofp",
    "unreachable",
    "urem",
    "va_arg",
    "void",
    "volatile",
    "xor",
    "zeroinitializer",
];

pub const KEYWORD_LENS: [u8; KEYWORD_COUNT] = compute_keyword_lens(&KEYWORD_NAMES);
pub const KW_MIN_LEN: usize = compute_kw_min_len(&KEYWORD_NAMES);
pub const KW_MAX_LEN: usize = compute_kw_max_len(&KEYWORD_NAMES);

const _: () = assert!(KEYWORD_NAMES.len() == KEYWORD_COUNT);
const _: () = assert!(KEYWORD_LENS.len() == KEYWORD_COUNT);
const _: () = assert!(KW_MIN_LEN >= 2 && KW_MAX_LEN <= 16);

#[inline(always)]
const fn kw_hash(len: usize, first: u8, b1: u8, k4: u8) -> usize {
    let val = len
        .wrapping_mul(732)
        .wrapping_add((first as usize).wrapping_mul(6809))
        .wrapping_add((b1 as usize).wrapping_mul(4076))
        .wrapping_add((k4 as usize).wrapping_mul(6850));
    (val ^ (val >> 1)) & KW_SLOT_MASK
}

const KW_TABLE: [u8; 256] = {
    let mut table = [KW_EMPTY; 256];
    let mut i = 0;
    while i < KEYWORD_COUNT {
        let name = KEYWORD_NAMES[i].as_bytes();
        let len = KEYWORD_LENS[i] as usize;
        let k4 = if len > 4 { name[4] } else { name[len - 1] };
        let h = kw_hash(len, name[0], name[1], k4);
        assert!(table[h] == KW_EMPTY);
        table[h] = i as u8;
        i += 1;
    }
    table
};

const TOK_IDENT: u8 = 3;
const TOK_SEMICOLON: u8 = 4;
const TOK_QUOTE: u8 = 5;

pub const TOKEN_KIND: [u8; 256] = make_token_kind(
    TOK_IDENT,
    &[
        (b'.', TOK_IDENT),
        (b';', TOK_SEMICOLON),
        (b'"', TOK_QUOTE),
    ],
);

pub const TOKEN_LUT_LO: [i8; 32] = make_token_lut_lo(&TOKEN_KIND);

#[inline(always)]
const fn len_matches(n: usize, idx: usize) -> bool {
    n == KEYWORD_LENS[idx] as usize
}

pub struct LlvmLexer;

impl LangLexer<KEYWORD_COUNT> for LlvmLexer {
    const TOKEN_KIND: [u8; 256] = TOKEN_KIND;
    const TOKEN_LUT_LO: [i8; 32] = TOKEN_LUT_LO;
    const HAS_DOLLAR_IDENT: bool = false;
    const SUPPORTS_TEMPLATES: bool = false;

    #[inline(always)]
    fn match_keyword(cand: &[u8]) -> Option<usize> {
        let n = cand.len();
        if n.wrapping_sub(KW_MIN_LEN) <= KW_MAX_LEN - KW_MIN_LEN {
            let k4 = if n > 4 { cand[4] } else { cand[n - 1] };
            let h = kw_hash(n, cand[0], cand[1], k4);
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
            TOK_SEMICOLON => finish_line_comment(content, i + 1, line_state, acc),
            TOK_QUOTE => handle_quoted_literal(content, i, b'"', line_state, acc),
            _ => {
                *line_state |= STATE_CODE;
                i + 1
            }
        }
    }
}

pub fn parse_into(content: &[u8], res: &mut ParseResult<KEYWORD_COUNT>) {
    unified_parse_into::<KEYWORD_COUNT, LlvmLexer>(content, res);
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_llvm_keyword_table_and_mph() {
        for (i, &name) in KEYWORD_NAMES.iter().enumerate() {
            let idx = LlvmLexer::match_keyword(name.as_bytes());
            assert_eq!(idx, Some(i), "keyword {} failed match", name);
        }
        assert_eq!(LlvmLexer::match_keyword(b"notakeyword"), None);
    }

    #[test]
    fn test_llvm_comments_and_strings() {
        let code = b"; LLVM IR module\ntarget datalayout = \"e-m:e-p270:32:32-p271:32:32\"\ntarget triple = \"x86_64-unknown-linux-gnu\"\n\ndefine i32 @main() {\nentry:\n  ; return statement\n  ret i32 0\n}\n";
        let mut res = ParseResult::default();
        parse_into(code, &mut res);
        assert_eq!(res.lines_processed, 9);
        assert_eq!(res.comment_lines, 2);
        assert_eq!(res.code_lines, 6);
        assert_eq!(res.blank_lines, 1);
        assert!(res.total_keywords >= 3);
    }
}
