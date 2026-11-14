use super::{
    compute_keyword_lens, compute_kw_max_len, compute_kw_min_len, finish_line_comment,
    handle_quoted_literal, kw_hash_fml_xor, make_kw_table_fml_xor, make_token_kind,
    make_token_lut_lo, match_keyword_caseless_with, unified_parse_into, LangLexer, ParseResult,
    STATE_CODE, TOK_CODE,
};

pub const KEYWORD_COUNT: usize = 44;
const KW_SLOT_MASK: usize = 127;

pub const KEYWORD_NAMES: [&str; KEYWORD_COUNT] = [
    "accept", "add", "call", "close", "compute", "copy", "data", "display",
    "divide", "division", "else", "end", "environment", "evaluate", "exit",
    "fd", "file", "goback", "identification", "if", "initialize", "inspect",
    "merge", "move", "multiply", "open", "perform", "pic", "procedure",
    "read", "release", "return", "rewrite", "search", "section", "set",
    "sort", "start", "stop", "string", "subtract", "unstring", "use",
    "when",
];

pub const KEYWORD_LENS: [u8; KEYWORD_COUNT] = compute_keyword_lens(&KEYWORD_NAMES);
pub const KW_MIN_LEN: usize = compute_kw_min_len(&KEYWORD_NAMES);
pub const KW_MAX_LEN: usize = compute_kw_max_len(&KEYWORD_NAMES);

#[inline(always)]
const fn kw_hash(first: u8, mid: u8, last: u8, len: usize) -> usize {
    kw_hash_fml_xor(first, mid, last, len, [1, 2, 29, 17], 0, KW_SLOT_MASK)
}

const KW_TABLE: [u8; 128] = make_kw_table_fml_xor(&KEYWORD_NAMES, [1, 2, 29, 17], 0);

const TOK_IDENT: u8 = 3;
const TOK_STAR: u8 = 4;
const TOK_SLASH: u8 = 5;
const TOK_QUOTE: u8 = 6;
const TOK_SQUOTE: u8 = 7;

pub const TOKEN_KIND: [u8; 256] = make_token_kind(
    TOK_IDENT,
    &[
        (b'_', TOK_CODE),
        (b'-', TOK_IDENT),
        (b'*', TOK_STAR),
        (b'/', TOK_SLASH),
        (b'"', TOK_QUOTE),
        (b'\'', TOK_SQUOTE),
    ],
);

pub const TOKEN_LUT_LO: [i8; 32] = make_token_lut_lo(&TOKEN_KIND);

pub struct CobolLexer;

impl LangLexer<KEYWORD_COUNT> for CobolLexer {
    const TOKEN_KIND: [u8; 256] = TOKEN_KIND;
    const TOKEN_LUT_LO: [i8; 32] = TOKEN_LUT_LO;
    const HAS_DOLLAR_IDENT: bool = false;
    const SUPPORTS_TEMPLATES: bool = false;
    const HAS_LINE_START_HOOK: bool = true;

    #[inline(always)]
    fn handle_line_start(
        content: &[u8],
        i: usize,
        line_state: &mut u8,
        acc: &mut ParseResult<KEYWORD_COUNT>,
    ) -> usize {
        let mut line_start = i;
        let min_start = i.saturating_sub(6);
        while line_start > min_start && content[line_start - 1] != b'\n' {
            line_start -= 1;
        }
        if (line_start == 0 || content[line_start - 1] == b'\n') && line_start + 6 < content.len() {
            let ind = content[line_start + 6];
            if (ind == b'*' || ind == b'/')
                && content[line_start..line_start + 6]
                    .iter()
                    .all(|&b| b == b' ' || b.is_ascii_digit())
            {
                return finish_line_comment(content, line_start + 7, line_state, acc);
            }
        }
        i
    }

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
            TOK_STAR => {
                if i + 1 < content_len && content[i + 1] == b'>' {
                    finish_line_comment(content, i + 1, line_state, acc)
                } else {
                    *line_state |= STATE_CODE;
                    i + 1
                }
            }
            TOK_SLASH => {
                *line_state |= STATE_CODE;
                i + 1
            }
            TOK_QUOTE => handle_quoted_literal(content, i, b'"', line_state, acc),
            TOK_SQUOTE => handle_quoted_literal(content, i, b'\'', line_state, acc),
            _ => {
                *line_state |= STATE_CODE;
                i + 1
            }
        }
    }
}

pub fn parse_into(content: &[u8], res: &mut ParseResult<KEYWORD_COUNT>) {
    unified_parse_into::<KEYWORD_COUNT, CobolLexer>(content, res);
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_cobol_keyword_table_and_mph() {
        for (i, &name) in KEYWORD_NAMES.iter().enumerate() {
            let idx = CobolLexer::match_keyword(name.as_bytes());
            assert_eq!(idx, Some(i), "keyword {} failed match", name);
            let upper = name.to_ascii_uppercase();
            let idx_upper = CobolLexer::match_keyword(upper.as_bytes());
            assert_eq!(idx_upper, Some(i), "keyword {} upper failed match", upper);
        }
        assert_eq!(CobolLexer::match_keyword(b"notakeyword"), None);
        assert_eq!(CobolLexer::match_keyword(b"x"), None);
    }

    #[test]
    fn test_cobol_comments_and_strings() {
        let code = b"      * COBOL column 7 comment
                     IDENTIFICATION DIVISION.
                     PROGRAM-ID. HELLO.
                     PROCEDURE DIVISION.
                         DISPLAY 'HELLO WORLD'. *> inline comment
                         STOP RUN.
";
        let mut res = ParseResult::default();
        parse_into(code, &mut res);
        assert_eq!(res.lines_processed, 6);
        assert_eq!(res.comment_lines, 1);
        assert_eq!(res.code_lines, 5);
        assert_eq!(res.blank_lines, 0);
        assert!(res.total_keywords >= 4);

        let fixed_code = b"       IDENTIFICATION DIVISION.
      *THIS IS A LINE 2 COMMENT
      /THIS IS A LINE 3 COMMENT
       PROGRAM-ID. HELLO.
";
        let mut fixed_res = ParseResult::default();
        parse_into(fixed_code, &mut fixed_res);
        assert_eq!(fixed_res.lines_processed, 4);
        assert_eq!(fixed_res.comment_lines, 2);
        assert_eq!(fixed_res.code_lines, 2);
    }
}
