use super::{
    compute_keyword_lens, compute_kw_max_len, compute_kw_min_len, count_newlines_avx2,
    find_byte_avx2, handle_c_slash_comments, handle_quoted_literal,
    handle_triple_or_quoted_literal, kw_match_2_to_16, make_token_kind, make_token_lut_lo,
    unified_parse_into, LangLexer, ParseResult, STATE_CODE,
};

pub const KEYWORD_COUNT: usize = 81;
const KW_SLOT_MASK: usize = 255;
const KW_EMPTY: u8 = 255;

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

const _: () = assert!(KEYWORD_NAMES.len() == KEYWORD_COUNT);
const _: () = assert!(KEYWORD_LENS.len() == KEYWORD_COUNT);

const TOK_IDENT: u8 = 3;
const TOK_SLASH: u8 = 4;
const TOK_QUOTE: u8 = 5;
const TOK_APOSTROPHE: u8 = 6;
const TOK_AT: u8 = 7;
const TOK_DOLLAR: u8 = 8;

#[inline(always)]
const fn kw_hash(first: u8, mid: u8, last: u8, len: usize) -> usize {
    let val = (first as usize)
        .wrapping_add((mid as usize).wrapping_mul(2355))
        .wrapping_add((last as usize).wrapping_mul(1899))
        .wrapping_add(len.wrapping_mul(131));
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
        let n = cand.len();
        if n.wrapping_sub(KW_MIN_LEN) <= KW_MAX_LEN - KW_MIN_LEN {
            let h = kw_hash(cand[0], cand[n / 2], cand[n - 1], n);
            let id = KW_TABLE[h];
            if id != KW_EMPTY {
                let idx = id as usize;
                debug_assert!(idx < KEYWORD_COUNT);
                if KEYWORD_LENS[idx] as usize == n
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
