use super::{
    compute_keyword_lens, compute_kw_max_len, compute_kw_min_len, handle_nested_2byte_comment,
    handle_quoted_literal, kw_hash_fml_add, make_kw_table_fml_add, make_token_kind,
    make_token_lut_lo, match_keyword_with, unified_parse_into, LangLexer, ParseResult, STATE_CODE,
};

pub const KEYWORD_COUNT: usize = 56;
const KW_SLOT_MASK: usize = 127;

pub const KEYWORD_NAMES: [&str; KEYWORD_COUNT] = [
    "and",
    "as",
    "asr",
    "assert",
    "begin",
    "class",
    "constraint",
    "do",
    "done",
    "downto",
    "else",
    "end",
    "exception",
    "external",
    "false",
    "for",
    "fun",
    "function",
    "functor",
    "if",
    "in",
    "include",
    "inherit",
    "initializer",
    "land",
    "lazy",
    "let",
    "lor",
    "lsl",
    "lsr",
    "lxor",
    "match",
    "method",
    "mod",
    "module",
    "mutable",
    "new",
    "nonrec",
    "object",
    "of",
    "open",
    "or",
    "private",
    "rec",
    "sig",
    "struct",
    "then",
    "to",
    "true",
    "try",
    "type",
    "val",
    "virtual",
    "when",
    "while",
    "with",
];

pub const KEYWORD_LENS: [u8; KEYWORD_COUNT] = compute_keyword_lens(&KEYWORD_NAMES);
pub const KW_MIN_LEN: usize = compute_kw_min_len(&KEYWORD_NAMES);
pub const KW_MAX_LEN: usize = compute_kw_max_len(&KEYWORD_NAMES);

const TOK_IDENT: u8 = 3;
const TOK_LPAREN: u8 = 4;
const TOK_QUOTE: u8 = 5;
const TOK_APOSTROPHE: u8 = 6;
const TOK_LBRACE: u8 = 7;

#[inline(always)]
const fn kw_hash(first: u8, mid: u8, last: u8, len: usize) -> usize {
    kw_hash_fml_add(first, mid, last, len, [5, 97, 35, 97], 1, KW_SLOT_MASK)
}

const KW_TABLE: [u8; 128] = make_kw_table_fml_add(&KEYWORD_NAMES, [5, 97, 35, 97], 1);

pub const TOKEN_KIND: [u8; 256] = make_token_kind(
    TOK_IDENT,
    &[
        (b'(', TOK_LPAREN),
        (b'{', TOK_LBRACE),
        (b'"', TOK_QUOTE),
        (b'\'', TOK_APOSTROPHE),
    ],
);

pub const TOKEN_LUT_LO: [i8; 32] = make_token_lut_lo(&TOKEN_KIND);

#[cfg(test)]
#[inline(always)]
pub fn is_ocaml_extension(name: &[u8]) -> bool {
    super::detect_language(name, super::LANG_ALL_MASK) == Some(super::LangId::OCaml)
}

pub struct OCamlLexer;

impl LangLexer<KEYWORD_COUNT> for OCamlLexer {
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
            TOK_LPAREN => {
                handle_nested_2byte_comment(content, i, *b"(*", *b"*)", line_state, acc)
            }
            TOK_LBRACE => {
                if i + 1 < content_len && (content[i + 1] == b'|' || content[i + 1].is_ascii_lowercase()) {
                    let mut bar = i + 1;
                    while bar < content_len && content[bar] != b'|' && content[bar] != b'\n' && bar - i < 32 {
                        bar += 1;
                    }
                    if bar < content_len && content[bar] == b'|' {
                        *line_state |= STATE_CODE;
                        let delim = &content[i + 1..bar];
                        let mut cur = bar + 1;
                        while cur < content_len {
                            if content[cur] == b'|'
                                && cur + 1 + delim.len() < content_len
                                && &content[cur + 1..cur + 1 + delim.len()] == delim
                                && content[cur + 1 + delim.len()] == b'}'
                            {
                                return cur + 2 + delim.len();
                            }
                            if content[cur] == b'\n' {
                                acc.code_lines += 1;
                                acc.lines_processed += 1;
                            }
                            cur += 1;
                        }
                        return content_len;
                    }
                }
                *line_state |= STATE_CODE;
                i + 1
            }
            TOK_QUOTE => handle_quoted_literal(content, i, b'"', line_state, acc),
            TOK_APOSTROPHE => handle_quoted_literal(content, i, b'\'', line_state, acc),
            _ => i + 1,
        }
    }
}

pub fn parse_into(content: &[u8], acc: &mut ParseResult<KEYWORD_COUNT>) {
    unified_parse_into::<KEYWORD_COUNT, OCamlLexer>(content, acc);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ocaml_extensions() {
        assert!(is_ocaml_extension(b"main.ml"));
        assert!(is_ocaml_extension(b"main.mli"));
        assert!(!is_ocaml_extension(b"main.m"));
    }

    #[test]
    fn test_ocaml_keyword_table_and_mph() {
        assert_eq!(KEYWORD_NAMES.len(), KEYWORD_COUNT);
        assert_eq!(KEYWORD_LENS.len(), KEYWORD_COUNT);
        for (i, name) in KEYWORD_NAMES.iter().enumerate() {
            let b = name.as_bytes();
            let h = kw_hash(b[0], b[b.len() / 2], b[b.len() - 1], b.len());
            assert_eq!(KW_TABLE[h], i as u8);
        }
    }

    #[test]
    fn test_ocaml_comments_and_strings() {
        let code = b"(* comment (* nested *) *)\nlet rec fact n =\n  if n <= 1 then 1\n  else n * fact (n - 1)\n";
        let mut acc = ParseResult::<KEYWORD_COUNT>::default();
        parse_into(code, &mut acc);
        assert_eq!(acc.comment_lines, 1);
        assert_eq!(acc.code_lines, 3);
        assert_eq!(acc.lines_processed, 4);
        assert_eq!(acc.total_keywords, 5);
    }
}
