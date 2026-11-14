use super::{
    compute_keyword_lens, compute_kw_max_len, compute_kw_min_len, finish_line_comment,
    handle_c_slash_comments, handle_quoted_literal, kw_hash_fml_add, make_kw_table_fml_add,
    make_token_kind, make_token_lut_lo, match_keyword_caseless_with, unified_parse_into, LangLexer,
    ParseResult, STATE_CODE,
};

pub const KEYWORD_COUNT: usize = 69;
const KW_SLOT_MASK: usize = 255;

pub const KEYWORD_NAMES: [&str; KEYWORD_COUNT] = [
    "abstract",
    "and",
    "array",
    "as",
    "break",
    "callable",
    "case",
    "catch",
    "class",
    "clone",
    "const",
    "continue",
    "declare",
    "default",
    "die",
    "do",
    "echo",
    "else",
    "elseif",
    "empty",
    "enddeclare",
    "endfor",
    "endforeach",
    "endif",
    "endswitch",
    "endwhile",
    "eval",
    "exit",
    "extends",
    "final",
    "finally",
    "fn",
    "for",
    "foreach",
    "function",
    "global",
    "goto",
    "if",
    "implements",
    "include",
    "include_once",
    "instanceof",
    "insteadof",
    "interface",
    "isset",
    "list",
    "match",
    "namespace",
    "new",
    "or",
    "print",
    "private",
    "protected",
    "public",
    "readonly",
    "require",
    "require_once",
    "return",
    "static",
    "switch",
    "throw",
    "trait",
    "try",
    "unset",
    "use",
    "var",
    "while",
    "xor",
    "yield",
];

pub const KEYWORD_LENS: [u8; KEYWORD_COUNT] = compute_keyword_lens(&KEYWORD_NAMES);
pub const KW_MIN_LEN: usize = compute_kw_min_len(&KEYWORD_NAMES);
pub const KW_MAX_LEN: usize = compute_kw_max_len(&KEYWORD_NAMES);

const TOK_IDENT: u8 = 3;
const TOK_SLASH: u8 = 4;
const TOK_HASH: u8 = 5;
const TOK_QUOTE: u8 = 6;
const TOK_APOSTROPHE: u8 = 7;
const TOK_BACKTICK: u8 = 8;

#[inline(always)]
const fn kw_hash(first: u8, mid: u8, last: u8, len: usize) -> usize {
    kw_hash_fml_add(first, mid, last, len, [1, 1, 163, 153], 1, KW_SLOT_MASK)
}

const KW_TABLE: [u8; 256] = make_kw_table_fml_add(&KEYWORD_NAMES, [1, 1, 163, 153], 1);

pub const TOKEN_KIND: [u8; 256] = make_token_kind(
    TOK_IDENT,
    &[
        (b'/', TOK_SLASH),
        (b'#', TOK_HASH),
        (b'"', TOK_QUOTE),
        (b'\'', TOK_APOSTROPHE),
        (b'`', TOK_BACKTICK),
    ],
);

pub const TOKEN_LUT_LO: [i8; 32] = make_token_lut_lo(&TOKEN_KIND);

#[cfg(test)]
#[inline(always)]
pub fn is_php_extension(name: &[u8]) -> bool {
    super::detect_language(name, super::LANG_ALL_MASK) == Some(super::LangId::Php)
}

pub struct PhpLexer;

impl LangLexer<KEYWORD_COUNT> for PhpLexer {
    const TOKEN_KIND: [u8; 256] = TOKEN_KIND;
    const TOKEN_LUT_LO: [i8; 32] = TOKEN_LUT_LO;
    const HAS_DOLLAR_IDENT: bool = false;
    const SUPPORTS_TEMPLATES: bool = false;
    const HAS_SPECIAL_IDENT: bool = true;
    const HAS_LINE_START_HOOK: bool = false;

    #[inline(always)]
    fn match_keyword(_cand: &[u8]) -> Option<usize> {
        None
    }

    #[inline(always)]
    fn handle_ident(
        content: &[u8],
        start: usize,
        end: usize,
        _line_state: &mut u8,
        acc: &mut ParseResult<KEYWORD_COUNT>,
    ) -> usize {
        let is_variable_or_property = if start > 0 {
            let p1 = content[start - 1];
            p1 == b'$' || (start >= 2 && content[start - 2] == b'-' && p1 == b'>')
        } else {
            false
        };

        if is_variable_or_property {
            return end;
        }

        let cand = &content[start..end];
        if let Some(idx) = match_keyword_caseless_with(
            cand,
            KW_MIN_LEN,
            KW_MAX_LEN,
            &KW_TABLE,
            &KEYWORD_LENS,
            &KEYWORD_NAMES,
            |c| {
                kw_hash(
                    c[0].to_ascii_lowercase(),
                    c[c.len() / 2].to_ascii_lowercase(),
                    c[c.len() - 1].to_ascii_lowercase(),
                    c.len(),
                )
            },
        ) {
            acc.keyword_counts[idx] += 1;
            acc.total_keywords += 1;
        }

        end
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
            TOK_SLASH => handle_c_slash_comments(content, i, line_state, acc),
            TOK_HASH => finish_line_comment(content, i + 1, line_state, acc),
            TOK_QUOTE | TOK_APOSTROPHE | TOK_BACKTICK => {
                handle_quoted_literal(content, i, content[i], line_state, acc)
            }
            _ => {
                *line_state |= STATE_CODE;
                i + 1
            }
        }
    }
}

pub fn parse_into(content: &[u8], acc: &mut ParseResult<KEYWORD_COUNT>) {
    unified_parse_into::<KEYWORD_COUNT, PhpLexer>(content, acc);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_php_extensions() {
        assert!(is_php_extension(b"test.php"));
        assert!(is_php_extension(b"index.phtml"));
        assert!(!is_php_extension(b"test.ph"));
    }

    #[test]
    fn test_php_keyword_table_and_mph() {
        assert_eq!(KEYWORD_NAMES.len(), KEYWORD_COUNT);
        assert_eq!(KEYWORD_LENS.len(), KEYWORD_COUNT);
        for (i, name) in KEYWORD_NAMES.iter().enumerate() {
            let b = name.as_bytes();
            let h = kw_hash(b[0], b[b.len() / 2], b[b.len() - 1], b.len());
            assert_eq!(KW_TABLE[h], i as u8);
        }
    }

    #[test]
    fn test_php_comments_and_strings() {
        let code = b"<?php\n// single line comment\n# hash comment\n/* multi\nline */\nfunction foo($bar) {\n    echo \"hello $bar\";\n    return true;\n}\n";
        let mut acc = ParseResult::<KEYWORD_COUNT>::default();
        parse_into(code, &mut acc);
        assert_eq!(acc.comment_lines, 4);
        assert_eq!(acc.code_lines, 5);
        assert_eq!(acc.lines_processed, 9);
        assert_eq!(acc.total_keywords, 3);
    }

    #[test]
    fn test_php_case_insensitive_keywords() {
        let code = b"ECHO 'test';\nRETURN;\n";
        let mut acc = ParseResult::<KEYWORD_COUNT>::default();
        parse_into(code, &mut acc);
        assert_eq!(acc.total_keywords, 2);
    }
}
