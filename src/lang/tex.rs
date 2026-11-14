use super::{
    compute_keyword_lens, compute_kw_max_len, compute_kw_min_len, finish_line_comment,
    handle_quoted_literal, kw_hash_fml_add, make_kw_table_fml_add, make_token_kind,
    make_token_lut_lo, match_keyword_with, unified_parse_into, LangLexer, ParseResult, STATE_CODE,
    TOK_CODE,
};

pub const KEYWORD_COUNT: usize = 21;
const KW_SLOT_MASK: usize = 63;

pub const KEYWORD_NAMES: [&str; KEYWORD_COUNT] = [
    "author",
    "begin",
    "caption",
    "cite",
    "date",
    "documentclass",
    "else",
    "em",
    "end",
    "fi",
    "include",
    "includegraphics",
    "input",
    "item",
    "label",
    "ref",
    "section",
    "textbf",
    "textit",
    "title",
    "usepackage",
];

pub const KEYWORD_LENS: [u8; KEYWORD_COUNT] = compute_keyword_lens(&KEYWORD_NAMES);
pub const KW_MIN_LEN: usize = compute_kw_min_len(&KEYWORD_NAMES);
pub const KW_MAX_LEN: usize = compute_kw_max_len(&KEYWORD_NAMES);

#[inline(always)]
const fn kw_hash(first: u8, mid: u8, last: u8, len: usize) -> usize {
    kw_hash_fml_add(first, mid, last, len, [6259, 4510, 8, 8551], 1, KW_SLOT_MASK)
}

const KW_TABLE: [u8; 64] = make_kw_table_fml_add(&KEYWORD_NAMES, [6259, 4510, 8, 8551], 1);

const TOK_IDENT: u8 = 3;
const TOK_PERCENT: u8 = 4;
const TOK_QUOTE: u8 = 5;

pub const TOKEN_KIND: [u8; 256] = make_token_kind(
    TOK_IDENT,
    &[
        (b'_', TOK_CODE),
        (b'\\', TOK_IDENT),
        (b'@', TOK_IDENT),
        (b'%', TOK_PERCENT),
        (b'"', TOK_QUOTE),
    ],
);

pub const TOKEN_LUT_LO: [i8; 32] = make_token_lut_lo(&TOKEN_KIND);

pub struct TexLexer;

impl LangLexer<KEYWORD_COUNT> for TexLexer {
    const TOKEN_KIND: [u8; 256] = TOKEN_KIND;
    const TOKEN_LUT_LO: [i8; 32] = TOKEN_LUT_LO;
    const HAS_DOLLAR_IDENT: bool = false;
    const SUPPORTS_TEMPLATES: bool = false;

    #[inline(always)]
    fn match_keyword(cand: &[u8]) -> Option<usize> {
        let cand = if !cand.is_empty() && cand[0] == b'\\' { &cand[1..] } else { cand };
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
            TOK_PERCENT => finish_line_comment(content, i + 1, line_state, acc),
            TOK_QUOTE => handle_quoted_literal(content, i, b'"', line_state, acc),
            _ => {
                *line_state |= STATE_CODE;
                i + 1
            }
        }
    }
}

pub fn parse_into(content: &[u8], res: &mut ParseResult<KEYWORD_COUNT>) {
    unified_parse_into::<KEYWORD_COUNT, TexLexer>(content, res);
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_tex_keyword_table_and_mph() {
        for (i, &name) in KEYWORD_NAMES.iter().enumerate() {
            let idx = TexLexer::match_keyword(name.as_bytes());
            assert_eq!(idx, Some(i), "keyword {} failed match", name);
            let mut slash_name = Vec::from(b"\\");
            slash_name.extend_from_slice(name.as_bytes());
            assert_eq!(TexLexer::match_keyword(&slash_name), Some(i));
        }
        assert_eq!(TexLexer::match_keyword(b"notakeyword"), None);
        assert_eq!(TexLexer::match_keyword(b"\\notakeyword"), None);
    }

    #[test]
    fn test_tex_comments_and_strings() {
        let code = b"% LaTeX document\n\\documentclass{article}\n\\usepackage{amsmath}\n\n% Main document\n\\begin{document}\n\\title{Sample Paper}\n\\author{Author Name}\n\\maketitle\n\n\\section{Introduction}\nThis is a sample document % inline comment\n\\end{document}\n";
        let mut res = ParseResult::default();
        parse_into(code, &mut res);
        assert_eq!(res.lines_processed, 13);
        assert_eq!(res.comment_lines, 2);
        assert_eq!(res.code_lines, 9);
        assert_eq!(res.blank_lines, 2);
        assert!(res.total_keywords >= 5);
    }
}
