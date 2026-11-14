use super::{
    compute_keyword_lens, compute_kw_max_len, compute_kw_min_len, count_newlines_avx2,
    find_byte_avx2, handle_c_slash_comments, handle_quoted_literal,
    handle_triple_or_quoted_literal, kw_hash_fml_add, make_kw_table_fml_add, make_token_kind,
    make_token_lut_lo, match_keyword_with, unified_parse_into, LangLexer, ParseResult, STATE_CODE,
};

pub const KEYWORD_COUNT: usize = 81;
const KW_SLOT_MASK: usize = 255;

pub const KEYWORD_NAMES: [&str; KEYWORD_COUNT] = [
    "abstract",
    "as",
    "base",
    "bool",
    "break",
    "byte",
    "case",
    "catch",
    "char",
    "checked",
    "class",
    "const",
    "continue",
    "decimal",
    "default",
    "delegate",
    "do",
    "double",
    "else",
    "enum",
    "event",
    "explicit",
    "extern",
    "false",
    "finally",
    "fixed",
    "float",
    "for",
    "foreach",
    "goto",
    "if",
    "implicit",
    "in",
    "init",
    "int",
    "interface",
    "internal",
    "is",
    "lock",
    "long",
    "namespace",
    "new",
    "null",
    "object",
    "operator",
    "out",
    "override",
    "params",
    "private",
    "protected",
    "public",
    "readonly",
    "record",
    "ref",
    "return",
    "sbyte",
    "sealed",
    "short",
    "sizeof",
    "stackalloc",
    "static",
    "string",
    "struct",
    "switch",
    "this",
    "throw",
    "true",
    "try",
    "typeof",
    "uint",
    "ulong",
    "unchecked",
    "unsafe",
    "ushort",
    "using",
    "var",
    "virtual",
    "void",
    "volatile",
    "while",
    "yield",
];

pub const KEYWORD_LENS: [u8; KEYWORD_COUNT] = compute_keyword_lens(&KEYWORD_NAMES);
pub const KW_MIN_LEN: usize = compute_kw_min_len(&KEYWORD_NAMES);
pub const KW_MAX_LEN: usize = compute_kw_max_len(&KEYWORD_NAMES);

const TOK_IDENT: u8 = 3;
const TOK_SLASH: u8 = 4;
const TOK_QUOTE: u8 = 5;
const TOK_APOSTROPHE: u8 = 6;
const TOK_AT: u8 = 7;
const TOK_DOLLAR: u8 = 8;

#[inline(always)]
const fn kw_hash(first: u8, mid: u8, last: u8, len: usize) -> usize {
    kw_hash_fml_add(first, mid, last, len, [1, 2355, 1899, 131], 1, KW_SLOT_MASK)
}

const KW_TABLE: [u8; 256] = make_kw_table_fml_add(&KEYWORD_NAMES, [1, 2355, 1899, 131], 1);

pub const TOKEN_KIND: [u8; 256] = make_token_kind(
    TOK_IDENT,
    &[
        (b'/', TOK_SLASH),
        (b'"', TOK_QUOTE),
        (b'\'', TOK_APOSTROPHE),
        (b'@', TOK_AT),
        (b'$', TOK_DOLLAR),
    ],
);

pub const TOKEN_LUT_LO: [i8; 32] = make_token_lut_lo(&TOKEN_KIND);

#[cfg(test)]
#[inline(always)]
pub fn is_csharp_extension(name: &[u8]) -> bool {
    super::detect_language(name, super::LANG_ALL_MASK) == Some(super::LangId::CSharp)
}

pub struct CSharpLexer;

impl LangLexer<KEYWORD_COUNT> for CSharpLexer {
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
            TOK_SLASH => handle_c_slash_comments(content, i, line_state, acc),
            TOK_QUOTE => handle_triple_or_quoted_literal(content, i, b'"', line_state, acc),
            TOK_AT => {
                *line_state |= STATE_CODE;
                if i + 1 < content_len && content[i + 1] == b'"' {
                    let mut j = i + 2;
                    while j < content_len {
                        let Some(pos) = find_byte_avx2(&content[j..], b'"') else {
                            j = content_len;
                            break;
                        };
                        let q = j + pos;
                        if q + 1 < content_len && content[q + 1] == b'"' {
                            j = q + 2;
                        } else {
                            j = q + 1;
                            break;
                        }
                    }
                    let nl = count_newlines_avx2(&content[i + 2..j]);
                    if nl > 0 {
                        acc.code_lines += nl as i64;
                        acc.lines_processed += nl as i64;
                    }
                    j
                } else {
                    i + 1
                }
            }
            TOK_DOLLAR => {
                *line_state |= STATE_CODE;
                if i + 1 < content_len && content[i + 1] == b'@' && i + 2 < content_len && content[i + 2] == b'"' {
                    let mut j = i + 3;
                    while j < content_len {
                        let Some(pos) = find_byte_avx2(&content[j..], b'"') else {
                            j = content_len;
                            break;
                        };
                        let q = j + pos;
                        if q + 1 < content_len && content[q + 1] == b'"' {
                            j = q + 2;
                        } else {
                            j = q + 1;
                            break;
                        }
                    }
                    let nl = count_newlines_avx2(&content[i + 3..j]);
                    if nl > 0 {
                        acc.code_lines += nl as i64;
                        acc.lines_processed += nl as i64;
                    }
                    j
                } else if i + 1 < content_len && content[i + 1] == b'"' {
                    handle_quoted_literal(content, i + 1, b'"', line_state, acc)
                } else {
                    i + 1
                }
            }
            TOK_APOSTROPHE => handle_quoted_literal(content, i, b'\'', line_state, acc),
            _ => {
                *line_state |= STATE_CODE;
                i + 1
            }
        }
    }
}

#[inline(always)]
pub fn parse_into(content: &[u8], acc: &mut ParseResult<KEYWORD_COUNT>) {
    unified_parse_into::<KEYWORD_COUNT, CSharpLexer>(content, acc);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csharp_keyword_table_and_mph() {
        for (i, &name) in KEYWORD_NAMES.iter().enumerate() {
            let b = name.as_bytes();
            let idx = CSharpLexer::match_keyword(b);
            assert_eq!(idx, Some(i), "keyword {}", name);
        }

        assert_eq!(CSharpLexer::match_keyword(b"notakeyword"), None);
        assert_eq!(CSharpLexer::match_keyword(b"x"), None);
        assert_eq!(CSharpLexer::match_keyword(b""), None);
    }

    #[test]
    fn test_csharp_extensions() {
        assert!(is_csharp_extension(b"Program.cs"));
        assert!(is_csharp_extension(b"PATH/TO/APP.CS"));
        assert!(!is_csharp_extension(b"Program.c"));
        assert!(!is_csharp_extension(b"Program.cpp"));
        assert!(!is_csharp_extension(b"Program.csx"));
    }

    #[test]
    fn test_csharp_lines_code_comments_blanks() {
        let src = b"namespace Example;\n\n// line comment\npublic class Program {\n    /* block\n       comment */\n    string s = @\"multi\nline\";\n}\n";
        let mut res = ParseResult::<KEYWORD_COUNT>::default();
        parse_into(src, &mut res);

        assert_eq!(res.lines_processed, 9);
        assert_eq!(res.blank_lines, 1);
        assert_eq!(res.comment_lines, 3);
        assert_eq!(res.code_lines, 5);
        assert_eq!(res.total_keywords, 4);
    }
}
