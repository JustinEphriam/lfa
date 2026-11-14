use super::{
    compute_keyword_lens, compute_kw_max_len, compute_kw_min_len, finish_line_comment,
    handle_quoted_literal, make_kw_table_b01ml_add, make_token_kind, make_token_lut_lo,
    match_keyword_caseless_with, unified_parse_into, LangLexer, ParseResult, STATE_CODE,
};

pub const KEYWORD_COUNT: usize = 44;
const KW_SLOT_MASK: usize = 127;

pub const KEYWORD_NAMES: [&str; KEYWORD_COUNT] = [
    "add_definitions",
    "add_dependencies",
    "add_executable",
    "add_library",
    "add_subdirectory",
    "add_test",
    "cmake_policy",
    "configure_file",
    "else",
    "elseif",
    "enable_language",
    "enable_testing",
    "endforeach",
    "endfunction",
    "endif",
    "endmacro",
    "endwhile",
    "execute_process",
    "export",
    "file",
    "find_file",
    "find_library",
    "find_package",
    "find_path",
    "find_program",
    "foreach",
    "function",
    "get_property",
    "if",
    "include",
    "install",
    "link_directories",
    "list",
    "macro",
    "mark_as_advanced",
    "message",
    "option",
    "project",
    "return",
    "set",
    "set_property",
    "string",
    "unset",
    "while",
];

pub const KEYWORD_LENS: [u8; KEYWORD_COUNT] = compute_keyword_lens(&KEYWORD_NAMES);
pub const KW_MIN_LEN: usize = compute_kw_min_len(&KEYWORD_NAMES);
pub const KW_MAX_LEN: usize = compute_kw_max_len(&KEYWORD_NAMES);

#[inline(always)]
const fn kw_hash(b0: u8, b1: u8, mid: u8, blast: u8, len: usize) -> usize {
    let v = (b0 as usize)
        .wrapping_mul(81)
        .wrapping_add((b1 as usize).wrapping_mul(172))
        .wrapping_add((mid as usize).wrapping_mul(59))
        .wrapping_add((blast as usize).wrapping_mul(108))
        .wrapping_add(len.wrapping_mul(93));
    (v ^ (v >> 5)) & KW_SLOT_MASK
}

const KW_TABLE: [u8; 128] = make_kw_table_b01ml_add(&KEYWORD_NAMES, [81, 172, 59, 108, 93], 5);

const TOK_IDENT: u8 = 3;
const TOK_HASH: u8 = 4;
const TOK_QUOTE: u8 = 5;

pub const TOKEN_KIND: [u8; 256] = make_token_kind(
    TOK_IDENT,
    &[
        (b'#', TOK_HASH),
        (b'"', TOK_QUOTE),
    ],
);

pub const TOKEN_LUT_LO: [i8; 32] = make_token_lut_lo(&TOKEN_KIND);

pub struct CMakeLexer;

impl LangLexer<KEYWORD_COUNT> for CMakeLexer {
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
            |c| kw_hash(c[0].to_ascii_lowercase(), c[1].to_ascii_lowercase(), c[c.len() / 2].to_ascii_lowercase(), c[c.len() - 1].to_ascii_lowercase(), c.len()),
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
            TOK_HASH => finish_line_comment(content, i + 1, line_state, acc),
            TOK_QUOTE => handle_quoted_literal(content, i, b'"', line_state, acc),
            _ => {
                *line_state |= STATE_CODE;
                i + 1
            }
        }
    }
}

pub fn parse_into(content: &[u8], res: &mut ParseResult<KEYWORD_COUNT>) {
    unified_parse_into::<KEYWORD_COUNT, CMakeLexer>(content, res);
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_cmake_keyword_table_and_mph() {
        for (i, &name) in KEYWORD_NAMES.iter().enumerate() {
            let idx = CMakeLexer::match_keyword(name.as_bytes());
            assert_eq!(idx, Some(i), "keyword {} failed match", name);
            let upper = name.to_ascii_uppercase();
            let idx_upper = CMakeLexer::match_keyword(upper.as_bytes());
            assert_eq!(idx_upper, Some(i), "keyword {} upper failed match", upper);
        }
        assert_eq!(CMakeLexer::match_keyword(b"notakeyword"), None);
        assert_eq!(CMakeLexer::match_keyword(b"x"), None);
    }

    #[test]
    fn test_cmake_comments_and_strings() {
        let cmake = b"# CMakeLists.txt\nproject(MyProject)\nadd_executable(app main.cpp)\nset(VAR \"hello\")\n";
        let mut res = ParseResult::default();
        parse_into(cmake, &mut res);
        assert_eq!(res.lines_processed, 4);
        assert_eq!(res.code_lines, 3);
        assert_eq!(res.comment_lines, 1);
        assert_eq!(res.blank_lines, 0);
        assert_eq!(res.total_keywords, 3);
    }
}
