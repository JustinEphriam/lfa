use super::{
    compute_keyword_lens, compute_kw_max_len, compute_kw_min_len, finish_line_comment,
    handle_char_or_lifetime, handle_nested_2byte_comment, handle_quoted_literal, kw_hash_fml_add,
    make_kw_table_fml_add, make_token_kind, make_token_lut_lo, match_keyword_with,
    unified_parse_into, LangLexer, ParseResult, STATE_CODE,
};

pub const KEYWORD_COUNT: usize = 34;
const KW_SLOT_MASK: usize = 63;

pub const KEYWORD_NAMES: [&str; KEYWORD_COUNT] = [
    "anyclass",
    "as",
    "case",
    "class",
    "data",
    "default",
    "deriving",
    "do",
    "else",
    "family",
    "forall",
    "foreign",
    "hiding",
    "if",
    "import",
    "in",
    "infix",
    "infixl",
    "infixr",
    "instance",
    "let",
    "mdo",
    "module",
    "newtype",
    "of",
    "pattern",
    "qualified",
    "role",
    "static",
    "stock",
    "then",
    "type",
    "via",
    "where",
];

pub const KEYWORD_LENS: [u8; KEYWORD_COUNT] = compute_keyword_lens(&KEYWORD_NAMES);
pub const KW_MIN_LEN: usize = compute_kw_min_len(&KEYWORD_NAMES);
pub const KW_MAX_LEN: usize = compute_kw_max_len(&KEYWORD_NAMES);

const TOK_IDENT: u8 = 3;
const TOK_MINUS: u8 = 4;
const TOK_LBRACE: u8 = 5;
const TOK_QUOTE: u8 = 6;
const TOK_APOSTROPHE: u8 = 7;

#[inline(always)]
const fn kw_hash(first: u8, mid: u8, last: u8, len: usize) -> usize {
    kw_hash_fml_add(first, mid, last, len, [1, 31, 11, 31], 1, KW_SLOT_MASK)
}

const KW_TABLE: [u8; 64] = make_kw_table_fml_add(&KEYWORD_NAMES, [1, 31, 11, 31], 1);

pub const TOKEN_KIND: [u8; 256] = make_token_kind(
    TOK_IDENT,
    &[
        (b'-', TOK_MINUS),
        (b'{', TOK_LBRACE),
        (b'"', TOK_QUOTE),
        (b'\'', TOK_APOSTROPHE),
    ],
);

pub const TOKEN_LUT_LO: [i8; 32] = make_token_lut_lo(&TOKEN_KIND);

#[cfg(test)]
#[inline(always)]
pub fn is_haskell_extension(name: &[u8]) -> bool {
    super::detect_language(name, super::LANG_ALL_MASK) == Some(super::LangId::Haskell)
}

pub struct HaskellLexer;

impl LangLexer<KEYWORD_COUNT> for HaskellLexer {
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
        let content_len = content.len();
        match kind {
            TOK_MINUS => {
                if i + 1 < content_len && content[i + 1] == b'-' {
                    finish_line_comment(content, i + 2, line_state, acc)
                } else {
                    *line_state |= STATE_CODE;
                    i + 1
                }
            }
            TOK_LBRACE => {
                handle_nested_2byte_comment(content, i, *b"{-", *b"-}", line_state, acc)
            }
            TOK_QUOTE => handle_quoted_literal(content, i, b'"', line_state, acc),
            TOK_APOSTROPHE => handle_char_or_lifetime(content, i, line_state),
            _ => {
                *line_state |= STATE_CODE;
                i + 1
            }
        }
    }
}

pub fn parse_into(content: &[u8], acc: &mut ParseResult<KEYWORD_COUNT>) {
    unified_parse_into::<KEYWORD_COUNT, HaskellLexer>(content, acc);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_haskell_extensions() {
        assert!(is_haskell_extension(b"main.hs"));
        assert!(is_haskell_extension(b"script.lhs"));
        assert!(!is_haskell_extension(b"main.h"));
    }

    #[test]
    fn test_haskell_keyword_table_and_mph() {
        assert_eq!(KEYWORD_NAMES.len(), KEYWORD_COUNT);
        assert_eq!(KEYWORD_LENS.len(), KEYWORD_COUNT);
        for (i, name) in KEYWORD_NAMES.iter().enumerate() {
            let b = name.as_bytes();
            let h = kw_hash(b[0], b[b.len() / 2], b[b.len() - 1], b.len());
            assert_eq!(KW_TABLE[h], i as u8);
        }
    }

    #[test]
    fn test_haskell_comments_and_strings() {
        let code = b"-- line comment\nmodule Main where\n{- nested {- block -} -}\nmain = putStrLn \"hello\"\n";
        let mut acc = ParseResult::<KEYWORD_COUNT>::default();
        parse_into(code, &mut acc);
        assert_eq!(acc.comment_lines, 2);
        assert_eq!(acc.code_lines, 2);
        assert_eq!(acc.lines_processed, 4);
        assert_eq!(acc.total_keywords, 2);
    }
}
