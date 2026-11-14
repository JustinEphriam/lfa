use super::{
    compute_keyword_lens, compute_kw_max_len, compute_kw_min_len, finish_line_comment,
    handle_c_slash_comments, handle_quoted_literal, kw_hash_fml_add, make_kw_table_fml_add,
    make_token_kind, make_token_lut_lo, match_keyword_caseless_with, unified_parse_into, LangLexer,
    ParseResult, STATE_CODE,
};

pub const KEYWORD_COUNT: usize = 34;
const KW_SLOT_MASK: usize = 255;

pub const KEYWORD_NAMES: [&str; KEYWORD_COUNT] = [
    "align",
    "bits",
    "bss",
    "byte",
    "data",
    "db",
    "dd",
    "dq",
    "dt",
    "dw",
    "dword",
    "endm",
    "endp",
    "equ",
    "extern",
    "global",
    "globl",
    "incbin",
    "include",
    "macro",
    "offset",
    "org",
    "proc",
    "ptr",
    "qword",
    "resb",
    "resd",
    "resq",
    "resw",
    "rodata",
    "section",
    "segment",
    "text",
    "word",
];

pub const KEYWORD_LENS: [u8; KEYWORD_COUNT] = compute_keyword_lens(&KEYWORD_NAMES);
pub const KW_MIN_LEN: usize = compute_kw_min_len(&KEYWORD_NAMES);
pub const KW_MAX_LEN: usize = compute_kw_max_len(&KEYWORD_NAMES);

#[inline(always)]
const fn kw_hash(first: u8, mid: u8, last: u8, len: usize) -> usize {
    kw_hash_fml_add(first, mid, last, len, [4768, 170, 3660, 2264], 2, KW_SLOT_MASK)
}

const KW_TABLE: [u8; 256] = make_kw_table_fml_add(&KEYWORD_NAMES, [4768, 170, 3660, 2264], 2);

const TOK_IDENT: u8 = 3;
const TOK_SEMICOLON: u8 = 4;
const TOK_HASH: u8 = 5;
const TOK_SLASH: u8 = 6;
const TOK_QUOTE: u8 = 7;
const TOK_APOSTROPHE: u8 = 8;

pub const TOKEN_KIND: [u8; 256] = make_token_kind(
    TOK_IDENT,
    &[
        (b'.', TOK_IDENT),
        (b';', TOK_SEMICOLON),
        (b'#', TOK_HASH),
        (b'/', TOK_SLASH),
        (b'"', TOK_QUOTE),
        (b'\'', TOK_APOSTROPHE),
    ],
);

pub const TOKEN_LUT_LO: [i8; 32] = make_token_lut_lo(&TOKEN_KIND);

pub struct AssemblyLexer;

impl LangLexer<KEYWORD_COUNT> for AssemblyLexer {
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
        match kind {
            TOK_SEMICOLON | TOK_HASH => finish_line_comment(content, i + 1, line_state, acc),
            TOK_SLASH => handle_c_slash_comments(content, i, line_state, acc),
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
    unified_parse_into::<KEYWORD_COUNT, AssemblyLexer>(content, res);
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_asm_keyword_table_and_mph() {
        for (i, &name) in KEYWORD_NAMES.iter().enumerate() {
            let idx = AssemblyLexer::match_keyword(name.as_bytes());
            assert_eq!(idx, Some(i), "keyword {} failed match", name);
            let upper = name.to_ascii_uppercase();
            let idx_upper = AssemblyLexer::match_keyword(upper.as_bytes());
            assert_eq!(idx_upper, Some(i), "keyword {} upper failed match", upper);
        }
        assert_eq!(AssemblyLexer::match_keyword(b"notakeyword"), None);
        assert_eq!(AssemblyLexer::match_keyword(b"x"), None);
    }

    #[test]
    fn test_asm_comments_and_strings() {
        let code = b"; Assembly source code\n\
                     section .text\n\
                     global _start\n\
                     _start:\n\
                         mov rax, 1 ; syscall sys_write\n\
                         # gas comment\n\
                         // c-style comment\n\
                         ret\n";
        let mut res = ParseResult::default();
        parse_into(code, &mut res);
        assert_eq!(res.lines_processed, 8);
        assert_eq!(res.comment_lines, 3);
        assert_eq!(res.code_lines, 5);
        assert_eq!(res.blank_lines, 0);
        assert!(res.total_keywords >= 2);
    }
}
