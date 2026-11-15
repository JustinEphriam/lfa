use super::{
    compute_keyword_lens, compute_kw_max_len, compute_kw_min_len, find_byte_avx2,
    finish_block_comment, finish_line_comment, handle_triple_or_quoted_literal, make_token_kind,
    make_token_lut_lo, match_keyword_with, unified_parse_into, LangLexer, ParseResult, STATE_CODE,
};

pub const KEYWORD_COUNT: usize = 46;
const KW_SLOT_MASK: usize = 127;
const KW_EMPTY: u8 = 255;

pub const KEYWORD_NAMES: [&str; KEYWORD_COUNT] = [
    "and",
    "break",
    "by",
    "catch",
    "class",
    "continue",
    "debugger",
    "default",
    "delete",
    "do",
    "else",
    "export",
    "extends",
    "false",
    "finally",
    "for",
    "if",
    "import",
    "in",
    "instanceof",
    "is",
    "isnt",
    "loop",
    "new",
    "no",
    "not",
    "null",
    "of",
    "off",
    "on",
    "or",
    "return",
    "super",
    "switch",
    "then",
    "this",
    "throw",
    "true",
    "try",
    "typeof",
    "undefined",
    "unless",
    "until",
    "when",
    "while",
    "yield",
];

pub const KEYWORD_LENS: [u8; KEYWORD_COUNT] = compute_keyword_lens(&KEYWORD_NAMES);
pub const KW_MIN_LEN: usize = compute_kw_min_len(&KEYWORD_NAMES);
pub const KW_MAX_LEN: usize = compute_kw_max_len(&KEYWORD_NAMES);

const _: () = assert!(KEYWORD_NAMES.len() == KEYWORD_COUNT);
const _: () = assert!(KEYWORD_LENS.len() == KEYWORD_COUNT);
const _: () = assert!(KW_MIN_LEN >= 2 && KW_MAX_LEN <= 10);

#[inline(always)]
const fn kw_hash(cand: &[u8], len: usize) -> usize {
    let b0 = cand[0] as usize;
    let bmid = cand[len / 2] as usize;
    let blast = cand[len - 1] as usize;
    let val = len.wrapping_mul(6084)
        .wrapping_add(b0.wrapping_mul(3765))
        .wrapping_add(bmid.wrapping_mul(3672))
        .wrapping_add(blast.wrapping_mul(944));
    (val ^ (val >> 3)) & KW_SLOT_MASK
}

const KW_TABLE: [u8; 128] = {
    let mut table = [KW_EMPTY; 128];
    let mut i = 0;
    while i < KEYWORD_COUNT {
        let name = KEYWORD_NAMES[i].as_bytes();
        let len = KEYWORD_LENS[i] as usize;
        assert!(name.len() == len);
        assert!(len >= KW_MIN_LEN && len <= KW_MAX_LEN);
        let h = kw_hash(name, len);
        assert!(table[h] == KW_EMPTY);
        table[h] = i as u8;
        i += 1;
    }
    table
};

const TOK_IDENT: u8 = 3;
const TOK_HASH: u8 = 4;
const TOK_QUOTE: u8 = 5;
const TOK_APOSTROPHE: u8 = 6;

pub const TOKEN_KIND: [u8; 256] = make_token_kind(
    TOK_IDENT,
    &[
        (b'#', TOK_HASH),
        (b'"', TOK_QUOTE),
        (b'\'', TOK_APOSTROPHE),
    ],
);

pub const TOKEN_LUT_LO: [i8; 32] = make_token_lut_lo(&TOKEN_KIND);

#[inline(always)]
fn find_triple_hash(slice: &[u8]) -> Option<usize> {
    let mut cur = 0;
    while let Some(pos) = find_byte_avx2(&slice[cur..], b'#') {
        let idx = cur + pos;
        if idx + 2 < slice.len() && slice[idx + 1] == b'#' && slice[idx + 2] == b'#' {
            return Some(idx);
        }
        cur = idx + 1;
    }
    None
}

#[cfg(test)]
#[inline(always)]
pub fn is_coffeescript_extension(name: &[u8]) -> bool {
    super::detect_language(name, super::LANG_ALL_MASK) == Some(super::LangId::CoffeeScript)
}

pub struct CoffeeScriptLexer;

impl LangLexer<KEYWORD_COUNT> for CoffeeScriptLexer {
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
            |c| kw_hash(c, c.len()),
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
            TOK_HASH => {
                if i + 2 < content_len && content[i + 1] == b'#' && content[i + 2] == b'#' {
                    let body_start = i + 3;
                    let end_i = match find_triple_hash(&content[body_start..]) {
                        Some(pos) => body_start + pos + 3,
                        None => content_len,
                    };
                    return finish_block_comment(content, body_start, end_i, line_state, acc);
                }
                finish_line_comment(content, i + 1, line_state, acc)
            }
            TOK_QUOTE => handle_triple_or_quoted_literal(content, i, b'"', line_state, acc),
            TOK_APOSTROPHE => handle_triple_or_quoted_literal(content, i, b'\'', line_state, acc),
            _ => {
                *line_state |= STATE_CODE;
                i + 1
            }
        }
    }
}

#[inline(always)]
pub fn parse_into(content: &[u8], acc: &mut ParseResult<KEYWORD_COUNT>) {
    unified_parse_into::<KEYWORD_COUNT, CoffeeScriptLexer>(content, acc);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyword_table_and_mph() {
        assert_eq!(KEYWORD_COUNT, 46);
        for i in 0..KEYWORD_COUNT {
            let kw = KEYWORD_NAMES[i];
            let len = kw.len();
            assert_eq!(len, KEYWORD_LENS[i] as usize);
            let bytes = kw.as_bytes();
            let h = kw_hash(bytes, len);
            assert_eq!(KW_TABLE[h], i as u8);

            let mut res = ParseResult::default();
            parse_into(bytes, &mut res);
            assert_eq!(res.total_keywords, 1);
            assert_eq!(res.keyword_counts[i], 1);
            assert_eq!(res.lines_processed, 1);
        }

        let non_keywords = ["classes", "functions", "switches", "yields"];
        for s in non_keywords {
            let mut res = ParseResult::default();
            parse_into(s.as_bytes(), &mut res);
            assert_eq!(res.total_keywords, 0);
        }
    }

    #[test]
    fn test_comments_and_strings() {
        let text = concat!(
            "# line comment\n",
            "###\n",
            "block comment\n",
            "###\n",
            "class Animal\n",
            "  msg: \"hello # not comment\"\n",
            "  run: -> alert msg # inline comment\n"
        );
        let mut res = ParseResult::default();
        parse_into(text.as_bytes(), &mut res);
        assert_eq!(res.lines_processed, 7);
        assert_eq!(res.comment_lines, 4);
        assert_eq!(res.code_lines, 3);
        assert_eq!(res.blank_lines, 0);
    }

    #[test]
    fn test_lines_code_comments_blanks() {
        let text = concat!(
            "\n",
            "# comment line\n",
            "\n",
            "square = (x) ->\n",
            "  ### multiline\n",
            "      block ###\n",
            "  x * x # inline comment\n",
            "\n"
        );
        let mut res = ParseResult::default();
        parse_into(text.as_bytes(), &mut res);
        assert_eq!(res.lines_processed, 8);
        assert_eq!(res.blank_lines, 3);
        assert_eq!(res.comment_lines, 3);
        assert_eq!(res.code_lines, 2);
    }

    #[test]
    fn test_extensions() {
        assert!(is_coffeescript_extension(b"index.coffee"));
        assert!(is_coffeescript_extension(b"config.cson"));
        assert!(!is_coffeescript_extension(b"main.rs"));
    }
}
