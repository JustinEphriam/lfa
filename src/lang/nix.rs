use super::{
    compute_keyword_lens, compute_kw_max_len, compute_kw_min_len, count_newlines_avx2,
    finish_line_comment, handle_c_block_comment, handle_quoted_literal, kw_hash_fml_add,
    make_kw_table_fml_add, make_token_kind, make_token_lut_lo, match_keyword_with,
    unified_parse_into, LangLexer, ParseResult, STATE_CODE,
};

pub const KEYWORD_COUNT: usize = 10;
const KW_SLOT_MASK: usize = 31;

pub const KEYWORD_NAMES: [&str; KEYWORD_COUNT] = [
    "assert",
    "else",
    "if",
    "in",
    "inherit",
    "let",
    "or",
    "rec",
    "then",
    "with",
];

pub const KEYWORD_LENS: [u8; KEYWORD_COUNT] = compute_keyword_lens(&KEYWORD_NAMES);
pub const KW_MIN_LEN: usize = compute_kw_min_len(&KEYWORD_NAMES);
pub const KW_MAX_LEN: usize = compute_kw_max_len(&KEYWORD_NAMES);

#[inline(always)]
const fn kw_hash(first: u8, mid: u8, last: u8, len: usize) -> usize {
    kw_hash_fml_add(first, mid, last, len, [1, 1, 1, 2], 0, KW_SLOT_MASK)
}

const KW_TABLE: [u8; 32] = make_kw_table_fml_add(&KEYWORD_NAMES, [1, 1, 1, 2], 0);

const TOK_IDENT: u8 = 3;
const TOK_HASH: u8 = 4;
const TOK_SLASH: u8 = 5;
const TOK_QUOTE: u8 = 6;
const TOK_SQUOTE: u8 = 7;

pub const TOKEN_KIND: [u8; 256] = make_token_kind(
    TOK_IDENT,
    &[
        (b'-', TOK_IDENT),
        (b'#', TOK_HASH),
        (b'/', TOK_SLASH),
        (b'"', TOK_QUOTE),
        (b'\'', TOK_SQUOTE),
    ],
);

pub const TOKEN_LUT_LO: [i8; 32] = make_token_lut_lo(&TOKEN_KIND);

pub struct NixLexer;

impl LangLexer<KEYWORD_COUNT> for NixLexer {
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
            TOK_HASH => finish_line_comment(content, i + 1, line_state, acc),
            TOK_SLASH => {
                if i + 1 < content_len && content[i + 1] == b'*' {
                    handle_c_block_comment(content, i + 2, line_state, acc)
                } else {
                    *line_state |= STATE_CODE;
                    i + 1
                }
            }
            TOK_QUOTE => handle_quoted_literal(content, i, b'"', line_state, acc),
            TOK_SQUOTE => {
                *line_state |= STATE_CODE;
                if i + 1 < content_len && content[i + 1] == b'\'' {
                    let mut curr = i + 2;
                    let mut closed = false;
                    while curr + 1 < content_len {
                        if content[curr] == b'\'' && content[curr + 1] == b'\'' {
                            curr += 2;
                            closed = true;
                            break;
                        }
                        curr += 1;
                    }
                    if !closed {
                        curr = content_len;
                    }
                    let nl = count_newlines_avx2(&content[i + 2..curr]);
                    if nl > 0 {
                        acc.code_lines += nl as i64;
                        acc.lines_processed += nl as i64;
                    }
                    curr
                } else {
                    i + 1
                }
            }
            _ => {
                *line_state |= STATE_CODE;
                i + 1
            }
        }
    }
}

pub fn parse_into(content: &[u8], res: &mut ParseResult<KEYWORD_COUNT>) {
    unified_parse_into::<KEYWORD_COUNT, NixLexer>(content, res);
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_nix_keyword_table_and_mph() {
        for (i, &name) in KEYWORD_NAMES.iter().enumerate() {
            let idx = NixLexer::match_keyword(name.as_bytes());
            assert_eq!(idx, Some(i), "keyword {} failed match", name);
        }
        assert_eq!(NixLexer::match_keyword(b"notakeyword"), None);
    }

    #[test]
    fn test_nix_comments_and_strings() {
        let code = b"# Nix derivation\nlet\n  pkgs = import <nixpkgs> {};\n  /* multi\n     line */\n  str = ''\n    hello world\n  '';\nin\n  pkgs.stdenv.mkDerivation {\n    name = \"demo\";\n  }\n";
        let mut res = ParseResult::default();
        parse_into(code, &mut res);
        assert_eq!(res.lines_processed, 12);
        assert_eq!(res.comment_lines, 3);
        assert_eq!(res.code_lines, 9);
        assert_eq!(res.blank_lines, 0);
        assert!(res.total_keywords >= 2);
    }
}
