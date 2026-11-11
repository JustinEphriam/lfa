use super::{
    compute_keyword_lens, compute_kw_max_len, compute_kw_min_len, count_newlines_avx2,
    find_byte_avx2, find_star_slash, kw_match_2_to_16, make_token_lut_lo, skip_literal,
    skip_triple_literal, unified_parse_into, LangLexer, ParseResult, STATE_CODE, STATE_COMMENT,
    TOK_CODE, TOK_NL, TOK_WS,
};

pub const KEYWORD_COUNT: usize = 55;
const KW_SLOT_MASK: usize = 255;
const KW_EMPTY: u8 = 255;

pub const KEYWORD_NAMES: [&str; KEYWORD_COUNT] = [
    "abstract",
    "assert",
    "boolean",
    "break",
    "byte",
    "case",
    "catch",
    "char",
    "class",
    "const",
    "continue",
    "default",
    "do",
    "double",
    "else",
    "enum",
    "extends",
    "false",
    "final",
    "finally",
    "float",
    "for",
    "goto",
    "if",
    "implements",
    "import",
    "instanceof",
    "int",
    "interface",
    "long",
    "native",
    "new",
    "null",
    "package",
    "private",
    "protected",
    "public",
    "record",
    "return",
    "short",
    "static",
    "strictfp",
    "super",
    "switch",
    "synchronized",
    "this",
    "throw",
    "throws",
    "transient",
    "true",
    "try",
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

#[inline(always)]
const fn kw_hash(first: u8, mid: u8, last: u8, len: usize) -> usize {
    let val = (first as usize)
        .wrapping_add((mid as usize).wrapping_mul(12))
        .wrapping_add((last as usize).wrapping_mul(2276))
        .wrapping_add(len.wrapping_mul(17));
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
    table[b'/' as usize] = TOK_SLASH;
    table[b'"' as usize] = TOK_QUOTE;
    table[b'\'' as usize] = TOK_APOSTROPHE;
    table
};

pub const TOKEN_LUT_LO: [i8; 32] = make_token_lut_lo(&TOKEN_KIND);

#[cfg(test)]
#[inline(always)]
pub fn is_java_extension(name: &[u8]) -> bool {
    super::detect_language(name, super::LANG_ALL_MASK) == Some(super::LangId::Java)
}

pub struct JavaLexer;

impl LangLexer<KEYWORD_COUNT> for JavaLexer {
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
            TOK_SLASH => {
                if i + 1 < content_len {
                    let next_b = content[i + 1];
                    if next_b == b'/' {
                        *line_state |= STATE_COMMENT;
                        let nl_pos = find_byte_avx2(&content[i + 2..], b'\n');
                        return nl_pos.map_or(content_len, |pos| i + 2 + pos);
                    } else if next_b == b'*' {
                        *line_state |= STATE_COMMENT;
                        if let Some(pos) = find_star_slash(&content[i + 2..]) {
                            let star_slash_pos = i + 2 + pos;
                            if let Some(first_nl) =
                                find_byte_avx2(&content[i + 2..star_slash_pos], b'\n')
                            {
                                if (*line_state & STATE_CODE) != 0 {
                                    acc.code_lines += 1;
                                } else {
                                    acc.comment_lines += 1;
                                }
                                acc.lines_processed += 1;
                                let first_nl_abs = i + 2 + first_nl;
                                let inter_nls = count_newlines_avx2(
                                    &content[first_nl_abs + 1..star_slash_pos],
                                );
                                acc.comment_lines += inter_nls as i64;
                                acc.lines_processed += inter_nls as i64;
                                *line_state = STATE_COMMENT;
                            }
                            return star_slash_pos + 2;
                        } else {
                            if let Some(first_nl) = find_byte_avx2(&content[i + 2..], b'\n') {
                                if (*line_state & STATE_CODE) != 0 {
                                    acc.code_lines += 1;
                                } else {
                                    acc.comment_lines += 1;
                                }
                                acc.lines_processed += 1;
                                let first_nl_abs = i + 2 + first_nl;
                                let rem_nls = count_newlines_avx2(&content[first_nl_abs + 1..]);
                                acc.comment_lines += rem_nls as i64;
                                acc.lines_processed += rem_nls as i64;
                                *line_state = STATE_COMMENT;
                            }
                            return content_len;
                        }
                    }
                }
                *line_state |= STATE_CODE;
                i + 1
            }
            TOK_QUOTE => {
                *line_state |= STATE_CODE;
                if i + 2 < content_len && content[i + 1] == b'"' && content[i + 2] == b'"' {
                    let end_i = skip_triple_literal(content, i + 3, b'"');
                    let nl = count_newlines_avx2(&content[i + 3..end_i]);
                    if nl > 0 {
                        acc.code_lines += nl as i64;
                        acc.lines_processed += nl as i64;
                    }
                    end_i
                } else {
                    let end_i = skip_literal(content, i + 1, b'"');
                    let nl = count_newlines_avx2(&content[i + 1..end_i]);
                    if nl > 0 {
                        acc.code_lines += nl as i64;
                        acc.lines_processed += nl as i64;
                    }
                    end_i
                }
            }
            TOK_APOSTROPHE => {
                *line_state |= STATE_CODE;
                let end_i = skip_literal(content, i + 1, b'\'');
                let nl = count_newlines_avx2(&content[i + 1..end_i]);
                if nl > 0 {
                    acc.code_lines += nl as i64;
                    acc.lines_processed += nl as i64;
                }
                end_i
            }
            _ => {
                *line_state |= STATE_CODE;
                i + 1
            }
        }
    }
}

#[inline(always)]
pub fn parse_into(content: &[u8], acc: &mut ParseResult<KEYWORD_COUNT>) {
    unified_parse_into::<KEYWORD_COUNT, JavaLexer>(content, acc);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_java_keyword_table_and_mph() {
        for (i, &name) in KEYWORD_NAMES.iter().enumerate() {
            let b = name.as_bytes();
            let idx = JavaLexer::match_keyword(b);
            assert_eq!(idx, Some(i), "keyword {}", name);
        }

        assert_eq!(JavaLexer::match_keyword(b"notakeyword"), None);
        assert_eq!(JavaLexer::match_keyword(b"x"), None);
        assert_eq!(JavaLexer::match_keyword(b"abstractx"), None);
        assert_eq!(JavaLexer::match_keyword(b""), None);
    }

    #[test]
    fn test_java_extensions() {
        assert!(is_java_extension(b"Main.java"));
        assert!(is_java_extension(b"path/to/Foo.JAVA"));
        assert!(!is_java_extension(b"Main.jav"));
        assert!(!is_java_extension(b"Main.class"));
        assert!(!is_java_extension(b"Main.jar"));
    }

    #[test]
    fn test_java_lines_code_comments_blanks() {
        let src = b"package com.example;\n\n// line comment\npublic class Foo {\n    /* block\n       comment */\n    final String s = \"hello\\nworld\";\n}\n";
        let mut res = ParseResult::<KEYWORD_COUNT>::default();
        parse_into(src, &mut res);

        assert_eq!(res.lines_processed, 8);
        assert_eq!(res.blank_lines, 1);
        assert_eq!(res.comment_lines, 3);
        assert_eq!(res.code_lines, 4);
        assert_eq!(res.total_keywords, 4);
    }

    #[test]
    fn test_java_text_blocks() {
        let src = b"class Test {\n    String block = \"\"\"\n        line 1\n        line 2\n        \"\"\";\n}\n";
        let mut res = ParseResult::<KEYWORD_COUNT>::default();
        parse_into(src, &mut res);

        assert_eq!(res.lines_processed, 6);
        assert_eq!(res.code_lines, 6);
        assert_eq!(res.comment_lines, 0);
        assert_eq!(res.blank_lines, 0);
    }
}
