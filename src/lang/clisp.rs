use super::{
    compute_keyword_lens, compute_kw_max_len, compute_kw_min_len, finish_line_comment,
    handle_nested_2byte_comment, handle_quoted_literal, kw_hash_fml_xor, make_kw_table_fml_xor,
    make_token_kind, make_token_lut_lo, match_keyword_caseless_with, unified_parse_into, LangLexer,
    ParseResult, STATE_CODE,
};

pub const KEYWORD_COUNT: usize = 24;
const KW_SLOT_MASK: usize = 63;

pub const KEYWORD_NAMES: [&str; KEYWORD_COUNT] = [
    "and", "car", "cdr", "cond", "cons", "defmacro", "defpackage", "defparameter",
    "defstruct", "defun", "defvar", "format", "lambda", "let", "loop", "nil",
    "or", "print", "progn", "quote", "return", "setq", "unless", "when",
];

pub const KEYWORD_LENS: [u8; KEYWORD_COUNT] = compute_keyword_lens(&KEYWORD_NAMES);
pub const KW_MIN_LEN: usize = compute_kw_min_len(&KEYWORD_NAMES);
pub const KW_MAX_LEN: usize = compute_kw_max_len(&KEYWORD_NAMES);

#[inline(always)]
const fn kw_hash(first: u8, mid: u8, last: u8, len: usize) -> usize {
    kw_hash_fml_xor(first, mid, last, len, [1, 5, 12, 17], 0, KW_SLOT_MASK)
}

const KW_TABLE: [u8; 64] = make_kw_table_fml_xor(&KEYWORD_NAMES, [1, 5, 12, 17], 0);

const TOK_IDENT: u8 = 3;
const TOK_SEMI: u8 = 4;
const TOK_QUOTE: u8 = 5;
const TOK_HASH: u8 = 6;

pub const TOKEN_KIND: [u8; 256] = make_token_kind(
    TOK_IDENT,
    &[
        (b'-', TOK_IDENT),
        (b'*', TOK_IDENT),
        (b';', TOK_SEMI),
        (b'"', TOK_QUOTE),
        (b'#', TOK_HASH),
    ],
);

pub const TOKEN_LUT_LO: [i8; 32] = make_token_lut_lo(&TOKEN_KIND);

pub struct CommonLispLexer;

impl LangLexer<KEYWORD_COUNT> for CommonLispLexer {
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
            TOK_SEMI => finish_line_comment(content, i + 1, line_state, acc),
            TOK_HASH => {
                handle_nested_2byte_comment(content, i, *b"#|", *b"|#", line_state, acc)
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
    unified_parse_into::<KEYWORD_COUNT, CommonLispLexer>(content, res);
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_clisp_keyword_table_and_mph() {
        for (i, &name) in KEYWORD_NAMES.iter().enumerate() {
            let idx = CommonLispLexer::match_keyword(name.as_bytes());
            assert_eq!(idx, Some(i), "keyword {} failed match", name);
            let upper = name.to_ascii_uppercase();
            let idx_upper = CommonLispLexer::match_keyword(upper.as_bytes());
            assert_eq!(idx_upper, Some(i), "keyword {} upper failed match", upper);
        }
        assert_eq!(CommonLispLexer::match_keyword(b"notakeyword"), None);
        assert_eq!(CommonLispLexer::match_keyword(b"x"), None);
    }

    #[test]
    fn test_clisp_comments_and_strings() {
        let code = b";; Common Lisp sample\n                     (defun hello ()\n                       (format t \"Hello, world!~%\")) ; greeting\n                     #| multi\n                        line |#\n";
        let mut res = ParseResult::default();
        parse_into(code, &mut res);
        assert_eq!(res.lines_processed, 5);
        assert_eq!(res.comment_lines, 3);
        assert_eq!(res.code_lines, 2);
        assert_eq!(res.blank_lines, 0);
        assert!(res.total_keywords >= 2);
    }
}
