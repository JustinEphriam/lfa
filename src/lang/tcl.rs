use super::{
    compute_keyword_lens, compute_kw_max_len, compute_kw_min_len, finish_line_comment,
    handle_quoted_literal, kw_hash_fml_add, make_kw_table_fml_add, make_token_kind,
    make_token_lut_lo, match_keyword_with, unified_parse_into, LangLexer, ParseResult, STATE_CODE,
    STATE_NONE,
};

pub const KEYWORD_COUNT: usize = 79;
const KW_SLOT_MASK: usize = 255;

pub const KEYWORD_NAMES: [&str; KEYWORD_COUNT] = [
    "after",
    "append",
    "apply",
    "array",
    "break",
    "catch",
    "chan",
    "clock",
    "close",
    "concat",
    "continue",
    "dict",
    "encoding",
    "eof",
    "error",
    "eval",
    "exec",
    "exit",
    "expr",
    "file",
    "flush",
    "for",
    "foreach",
    "format",
    "gets",
    "glob",
    "global",
    "history",
    "if",
    "incr",
    "info",
    "interp",
    "join",
    "lappend",
    "lassign",
    "lindex",
    "linsert",
    "list",
    "llength",
    "lmap",
    "load",
    "lrange",
    "lrepeat",
    "lreplace",
    "lreverse",
    "lsearch",
    "lset",
    "lsort",
    "namespace",
    "open",
    "package",
    "pid",
    "proc",
    "puts",
    "pwd",
    "read",
    "regexp",
    "regsub",
    "rename",
    "return",
    "scan",
    "seek",
    "set",
    "socket",
    "source",
    "split",
    "string",
    "subst",
    "switch",
    "tell",
    "time",
    "trace",
    "unset",
    "update",
    "uplevel",
    "upvar",
    "variable",
    "vwait",
    "while",
];

pub const KEYWORD_LENS: [u8; KEYWORD_COUNT] = compute_keyword_lens(&KEYWORD_NAMES);
pub const KW_MIN_LEN: usize = compute_kw_min_len(&KEYWORD_NAMES);
pub const KW_MAX_LEN: usize = compute_kw_max_len(&KEYWORD_NAMES);

#[inline(always)]
const fn kw_hash(first: u8, mid: u8, last: u8, len: usize) -> usize {
    kw_hash_fml_add(first, mid, last, len, [6803, 6834, 7436, 741], 1, KW_SLOT_MASK)
}

const KW_TABLE: [u8; 256] = make_kw_table_fml_add(&KEYWORD_NAMES, [6803, 6834, 7436, 741], 1);

const TOK_IDENT: u8 = 3;
const TOK_HASH: u8 = 4;
const TOK_QUOTE: u8 = 5;

pub const TOKEN_KIND: [u8; 256] = make_token_kind(
    TOK_IDENT,
    &[
        (b':', TOK_IDENT),
        (b'#', TOK_HASH),
        (b'"', TOK_QUOTE),
    ],
);

pub const TOKEN_LUT_LO: [i8; 32] = make_token_lut_lo(&TOKEN_KIND);

pub struct TclLexer;

impl LangLexer<KEYWORD_COUNT> for TclLexer {
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
            TOK_HASH => {
                if *line_state == STATE_NONE {
                    finish_line_comment(content, i + 1, line_state, acc)
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
    unified_parse_into::<KEYWORD_COUNT, TclLexer>(content, res);
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_tcl_keyword_table_and_mph() {
        for (i, &name) in KEYWORD_NAMES.iter().enumerate() {
            let idx = TclLexer::match_keyword(name.as_bytes());
            assert_eq!(idx, Some(i), "keyword {} failed match", name);
        }
        assert_eq!(TclLexer::match_keyword(b"notakeyword"), None);
        assert_eq!(TclLexer::match_keyword(b"x"), None);
    }

    #[test]
    fn test_tcl_comments_and_strings() {
        let code = b"# Tcl script\n\
                     proc add {a b} {\n\
                         # addition\n\
                         return [expr {$a + $b}]\n\
                     }\n\
                     puts \"Result: [add 1 2]\"\n";
        let mut res = ParseResult::default();
        parse_into(code, &mut res);
        assert_eq!(res.lines_processed, 6);
        assert_eq!(res.comment_lines, 2);
        assert_eq!(res.code_lines, 4);
        assert_eq!(res.blank_lines, 0);
        assert!(res.total_keywords >= 4);
    }
}
