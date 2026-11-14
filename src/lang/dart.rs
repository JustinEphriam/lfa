use super::{
    compute_keyword_lens, compute_kw_max_len, compute_kw_min_len, count_newlines_avx2,
    handle_nested_c_slash_comments, handle_triple_or_quoted_literal, kw_match_2_to_16,
    make_token_lut_lo, skip_triple_literal, unified_parse_into, LangLexer, ParseResult, STATE_CODE,
    TOK_CODE, TOK_NL, TOK_WS,
};

pub const KEYWORD_COUNT: usize = 67;
const KW_SLOT_MASK: usize = 255;
const KW_EMPTY: u8 = 255;

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

const _: () = assert!(KEYWORD_NAMES.len() == KEYWORD_COUNT);
const _: () = assert!(KEYWORD_LENS.len() == KEYWORD_COUNT);

const TOK_IDENT: u8 = 3;
const TOK_SLASH: u8 = 4;
const TOK_QUOTE: u8 = 5;
const TOK_APOSTROPHE: u8 = 6;
const TOK_RAW_PREFIX: u8 = 7;

#[inline(always)]
const fn kw_hash(first: u8, mid: u8, last: u8, len: usize) -> usize {
    let v = (first as usize)
        .wrapping_mul(1)
        .wrapping_add((mid as usize).wrapping_mul(17))
        .wrapping_add((last as usize).wrapping_mul(91))
        .wrapping_add(len.wrapping_mul(85));
    (v ^ (v >> 1)) & KW_SLOT_MASK
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

pub const TOKEN_KIND: [u8; 256] = {
    let mut table = [TOK_CODE; 256];
    table[b' ' as usize] = TOK_WS;
    table[b'\t' as usize] = TOK_WS;
    table[b'\r' as usize] = TOK_WS;
    table[0x0b_usize] = TOK_WS;
    table[0x0c_usize] = TOK_WS;
    table[b'\n' as usize] = TOK_NL;
    let mut c = b'a';
    while c <= b'z' {
        table[c as usize] = TOK_IDENT;
        c += 1;
    }
    let mut c = b'A';
    while c <= b'Z' {
        table[c as usize] = TOK_IDENT;
        c += 1;
    }
    table[b'_' as usize] = TOK_IDENT;
    table[b'$' as usize] = TOK_IDENT;
    table[b'r' as usize] = TOK_RAW_PREFIX;
    table[b'/' as usize] = TOK_SLASH;
    table[b'"' as usize] = TOK_QUOTE;
    table[b'\'' as usize] = TOK_APOSTROPHE;
    table
};

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
        let cand = &content[start..end];
        let cand_len = cand.len();
        if (KW_MIN_LEN..=KW_MAX_LEN).contains(&cand_len) {
            let h = kw_hash(cand[0], cand[cand_len / 2], cand[cand_len - 1], cand_len);
            let idx = KW_TABLE[h] as usize;
            if idx < KEYWORD_COUNT && kw_match_2_to_16(cand, KEYWORD_NAMES[idx].as_bytes()) {
                acc.keyword_counts[idx] += 1;
                acc.total_keywords += 1;
            }
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
                    let end = super::skip_js_ident_cont_avx2(content, i + 1);
                    Self::handle_ident(content, i, end, line_state, acc)
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
