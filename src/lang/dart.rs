use super::{
    compute_keyword_lens, compute_kw_max_len, compute_kw_min_len, count_newlines_avx2,
    handle_nested_c_slash_comments, handle_triple_or_quoted_literal, kw_hash_fml_add,
    make_kw_table_fml_add, make_token_kind, make_token_lut_lo, match_keyword_with,
    skip_triple_literal, unified_parse_into, LangLexer, ParseResult, STATE_CODE,
};

pub const KEYWORD_COUNT: usize = 67;
const KW_SLOT_MASK: usize = 255;

pub const KEYWORD_NAMES: [&str; KEYWORD_COUNT] = [
    "abstract",
    "as",
    "assert",
    "async",
    "await",
    "base",
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "covariant",
    "default",
    "deferred",
    "do",
    "dynamic",
    "else",
    "enum",
    "export",
    "extends",
    "extension",
    "external",
    "factory",
    "false",
    "final",
    "finally",
    "for",
    "function",
    "get",
    "hide",
    "if",
    "implements",
    "import",
    "in",
    "interface",
    "is",
    "late",
    "library",
    "mixin",
    "new",
    "null",
    "of",
    "on",
    "operator",
    "part",
    "required",
    "rethrow",
    "return",
    "sealed",
    "set",
    "show",
    "static",
    "super",
    "switch",
    "sync",
    "this",
    "throw",
    "true",
    "try",
    "typedef",
    "var",
    "void",
    "when",
    "while",
    "with",
    "yield",
];

pub const KEYWORD_LENS: [u8; KEYWORD_COUNT] = compute_keyword_lens(&KEYWORD_NAMES);
pub const KW_MIN_LEN: usize = compute_kw_min_len(&KEYWORD_NAMES);
pub const KW_MAX_LEN: usize = compute_kw_max_len(&KEYWORD_NAMES);

const TOK_IDENT: u8 = 3;
const TOK_SLASH: u8 = 4;
const TOK_QUOTE: u8 = 5;
const TOK_APOSTROPHE: u8 = 6;
const TOK_RAW_PREFIX: u8 = 7;

#[inline(always)]
const fn kw_hash(first: u8, mid: u8, last: u8, len: usize) -> usize {
    kw_hash_fml_add(first, mid, last, len, [1, 17, 91, 85], 1, KW_SLOT_MASK)
}

const KW_TABLE: [u8; 256] = make_kw_table_fml_add(&KEYWORD_NAMES, [1, 17, 91, 85], 1);

pub const TOKEN_KIND: [u8; 256] = make_token_kind(
    TOK_IDENT,
    &[
        (b'$', TOK_IDENT),
        (b'r', TOK_RAW_PREFIX),
        (b'/', TOK_SLASH),
        (b'"', TOK_QUOTE),
        (b'\'', TOK_APOSTROPHE),
    ],
);

pub const TOKEN_LUT_LO: [i8; 32] = make_token_lut_lo(&TOKEN_KIND);

#[cfg(test)]
#[inline(always)]
pub fn is_dart_extension(name: &[u8]) -> bool {
    super::detect_language(name, super::LANG_ALL_MASK) == Some(super::LangId::Dart)
}

pub struct DartLexer;

impl LangLexer<KEYWORD_COUNT> for DartLexer {
    const TOKEN_KIND: [u8; 256] = TOKEN_KIND;
    const TOKEN_LUT_LO: [i8; 32] = TOKEN_LUT_LO;
    const HAS_DOLLAR_IDENT: bool = true;
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
            TOK_SLASH => handle_nested_c_slash_comments(content, i, line_state, acc),
            TOK_RAW_PREFIX => {
                if i + 1 < content_len && (content[i + 1] == b'"' || content[i + 1] == b'\'') {
                    *line_state |= STATE_CODE;
                    let quote = content[i + 1];
                    if i + 3 < content_len && content[i + 2] == quote && content[i + 3] == quote {
                        let end_i = skip_triple_literal(content, i + 4, quote);
                        let nl = count_newlines_avx2(&content[i + 4..end_i]);
                        if nl > 0 {
                            acc.code_lines += nl as i64;
                            acc.lines_processed += nl as i64;
                        }
                        end_i
                    } else {
                        let mut cur = i + 2;
                        while cur < content_len && content[cur] != quote && content[cur] != b'\n' {
                            cur += 1;
                        }
                        if cur < content_len && content[cur] == quote {
                            cur + 1
                        } else {
                            cur
                        }
                    }
                } else {
                    *line_state |= STATE_CODE;
                    let end = super::skip_js_ident_cont_avx2(content, i + 1);
                    if let Some(idx) = Self::match_keyword(&content[i..end]) {
                        acc.keyword_counts[idx] += 1;
                        acc.total_keywords += 1;
                    }
                    end
                }
            }
            TOK_QUOTE => handle_triple_or_quoted_literal(content, i, b'"', line_state, acc),
            TOK_APOSTROPHE => handle_triple_or_quoted_literal(content, i, b'\'', line_state, acc),
            _ => i + 1,
        }
    }
}

pub fn parse_into(content: &[u8], acc: &mut ParseResult<KEYWORD_COUNT>) {
    unified_parse_into::<KEYWORD_COUNT, DartLexer>(content, acc);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dart_extensions() {
        assert!(is_dart_extension(b"main.dart"));
        assert!(!is_dart_extension(b"main.dar"));
    }

    #[test]
    fn test_dart_keyword_table_and_mph() {
        assert_eq!(KEYWORD_NAMES.len(), KEYWORD_COUNT);
        assert_eq!(KEYWORD_LENS.len(), KEYWORD_COUNT);
        for (i, name) in KEYWORD_NAMES.iter().enumerate() {
            let b = name.as_bytes();
            let h = kw_hash(b[0], b[b.len() / 2], b[b.len() - 1], b.len());
            assert_eq!(KW_TABLE[h], i as u8);
        }
    }

    #[test]
    fn test_dart_comments_and_strings() {
        let code = b"// single comment\nvoid main() {\n  /* nested /* comment */ */\n  var s = r'''multi\nraw''';\n  print(\"hello\");\n}\n";
        let mut acc = ParseResult::<KEYWORD_COUNT>::default();
        parse_into(code, &mut acc);
        assert_eq!(acc.comment_lines, 2);
        assert_eq!(acc.code_lines, 5);
        assert_eq!(acc.lines_processed, 7);
        assert_eq!(acc.total_keywords, 2);
    }
}
