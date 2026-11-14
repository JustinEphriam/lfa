use super::{
    compute_keyword_lens, compute_kw_max_len, compute_kw_min_len, finish_line_comment,
    handle_quoted_literal, kw_match_2_to_16, make_token_kind, make_token_lut_lo,
    unified_parse_into, LangLexer, ParseResult, STATE_CODE, STATE_NONE,
};

pub const KEYWORD_COUNT: usize = 79;
const KW_SLOT_MASK: usize = 255;
const KW_EMPTY: u8 = 255;

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

const _: () = assert!(KEYWORD_NAMES.len() == KEYWORD_COUNT);
const _: () = assert!(KEYWORD_LENS.len() == KEYWORD_COUNT);
const _: () = assert!(KW_MIN_LEN >= 2 && KW_MAX_LEN <= 16);

#[inline(always)]
const fn kw_hash(first: u8, mid: u8, last: u8, len: usize) -> usize {
    let val = (first as usize)
        .wrapping_mul(6803)
        .wrapping_add((mid as usize).wrapping_mul(6834))
        .wrapping_add((last as usize).wrapping_mul(7436))
        .wrapping_add(len.wrapping_mul(741));
    (val ^ (val >> 1)) & KW_SLOT_MASK
}

const KW_TABLE: [u8; 256] = {
    let mut table = [KW_EMPTY; 256];
    let mut i = 0;
    while i < KEYWORD_COUNT {
        let name = KEYWORD_NAMES[i].as_bytes();
        let len = KEYWORD_LENS[i] as usize;
        assert!(name.len() == len);
        assert!(len >= KW_MIN_LEN && len <= KW_MAX_LEN);
        let h = kw_hash(name[0], name[len / 2], name[len - 1], len);
        assert!(table[h] == KW_EMPTY);
        table[h] = i as u8;
        i += 1;
    }
    table
};

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

#[inline(always)]
const fn len_matches(n: usize, idx: usize) -> bool {
    n == KEYWORD_LENS[idx] as usize
}

pub struct TclLexer;

impl LangLexer<KEYWORD_COUNT> for TclLexer {
    const TOKEN_KIND: [u8; 256] = TOKEN_KIND;
    const TOKEN_LUT_LO: [i8; 32] = TOKEN_LUT_LO;
    const HAS_DOLLAR_IDENT: bool = false;
    const SUPPORTS_TEMPLATES: bool = false;

    #[inline(always)]
    fn match_keyword(cand: &[u8]) -> Option<usize> {
        let n = cand.len();
        if n.wrapping_sub(KW_MIN_LEN) <= KW_MAX_LEN - KW_MIN_LEN {
            let h = kw_hash(cand[0], cand[n / 2], cand[n - 1], n);
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
