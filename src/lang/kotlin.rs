use super::{
    compute_keyword_lens, compute_kw_max_len, compute_kw_min_len, count_newlines_avx2,
    find_byte_avx2, kw_match_2_to_16, make_token_lut_lo, skip_literal,
    skip_nested_block_comment, unified_parse_into, LangLexer, ParseResult, STATE_CODE,
    STATE_COMMENT, TOK_CODE, TOK_NL, TOK_WS,
};

pub const KEYWORD_COUNT: usize = 74;
const KW_SLOT_MASK: usize = 255;
const KW_EMPTY: u8 = 255;

pub const KEYWORD_NAMES: [&str; KEYWORD_COUNT] = [
    "actual",
    "abstract",
    "annotation",
    "as",
    "break",
    "by",
    "catch",
    "class",
    "companion",
    "const",
    "constructor",
    "continue",
    "crossinline",
    "data",
    "delegate",
    "do",
    "dynamic",
    "else",
    "enum",
    "expect",
    "external",
    "false",
    "field",
    "file",
    "final",
    "finally",
    "for",
    "fun",
    "get",
    "if",
    "import",
    "in",
    "infix",
    "init",
    "inline",
    "inner",
    "internal",
    "is",
    "lateinit",
    "noinline",
    "null",
    "object",
    "open",
    "operator",
    "out",
    "override",
    "package",
    "param",
    "private",
    "property",
    "protected",
    "public",
    "receiver",
    "reified",
    "return",
    "sealed",
    "set",
    "setparam",
    "super",
    "suspend",
    "tailrec",
    "this",
    "throw",
    "true",
    "try",
    "typealias",
    "typeof",
    "val",
    "value",
    "var",
    "vararg",
    "when",
    "where",
    "while",
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
    let v = (first as usize)
        .wrapping_mul(1)
        .wrapping_add((mid as usize).wrapping_mul(3))
        .wrapping_add((last as usize).wrapping_mul(389))
        .wrapping_add(len.wrapping_mul(509));
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
    table[b'/' as usize] = TOK_SLASH;
    table[b'"' as usize] = TOK_QUOTE;
    table[b'\'' as usize] = TOK_APOSTROPHE;
    table
};

pub const TOKEN_LUT_LO: [i8; 32] = make_token_lut_lo(&TOKEN_KIND);

#[cfg(test)]
#[inline(always)]
pub fn is_kotlin_extension(name: &[u8]) -> bool {
    super::detect_language(name, super::LANG_ALL_MASK) == Some(super::LangId::Kotlin)
}

pub struct KotlinLexer;

impl LangLexer<KEYWORD_COUNT> for KotlinLexer {
    const TOKEN_KIND: [u8; 256] = TOKEN_KIND;
    const TOKEN_LUT_LO: [i8; 32] = TOKEN_LUT_LO;
    const HAS_DOLLAR_IDENT: bool = false;
    const SUPPORTS_TEMPLATES: bool = false;
    const HAS_SPECIAL_IDENT: bool = false;
    const HAS_LINE_START_HOOK: bool = false;

    #[inline(always)]
    fn match_keyword(cand: &[u8]) -> Option<usize> {
        let n = cand.len();
        if (KW_MIN_LEN..=KW_MAX_LEN).contains(&n) {
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
        mut i: usize,
        line_state: &mut u8,
        acc: &mut ParseResult<KEYWORD_COUNT>,
    ) -> usize {
        let content_len = content.len();
        match kind {
            TOK_SLASH => {
                if i + 1 < content_len {
                    let next = content[i + 1];
                    if next == b'/' {
                        *line_state |= STATE_COMMENT;
                        if let Some(pos) = find_byte_avx2(&content[i + 2..], b'\n') {
                            if (*line_state & STATE_CODE) != 0 {
                                acc.code_lines += 1;
                            } else {
                                acc.comment_lines += 1;
                            }
                            acc.lines_processed += 1;
                            *line_state = 0;
                            return i + 2 + pos + 1;
                        } else {
                            return content_len;
                        }
                    } else if next == b'*' {
                        *line_state |= STATE_COMMENT;
                        let end_comment = skip_nested_block_comment(content, i + 2);
                        let nls = count_newlines_avx2(&content[i..end_comment]);
                        if nls > 0 {
                            if (*line_state & STATE_CODE) != 0 {
                                acc.code_lines += 1;
                                acc.comment_lines += (nls - 1) as i64;
                            } else {
                                acc.comment_lines += nls as i64;
                            }
                            acc.lines_processed += nls as i64;
                            *line_state = STATE_COMMENT;
                        }
                        return end_comment;
                    }
                }
                *line_state |= STATE_CODE;
                i + 1
            }
            TOK_QUOTE => {
                *line_state |= STATE_CODE;
                if i + 2 < content_len && content[i + 1] == b'"' && content[i + 2] == b'"' {
                    i += 3;
                    while i + 2 < content_len {
                        if content[i] == b'"' && content[i + 1] == b'"' && content[i + 2] == b'"' {
                            i += 3;
                            break;
                        }
                        if content[i] == b'\n' {
                            acc.code_lines += 1;
                            acc.lines_processed += 1;
                            *line_state = STATE_CODE;
                        }
                        i += 1;
                    }
                    if i + 2 >= content_len && i < content_len {
                        let nl = count_newlines_avx2(&content[i..]);
                        acc.code_lines += nl as i64;
                        acc.lines_processed += nl as i64;
                        i = content_len;
                    }
                    i
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

pub fn parse_into(content: &[u8], acc: &mut ParseResult<KEYWORD_COUNT>) {
    unified_parse_into::<KEYWORD_COUNT, KotlinLexer>(content, acc);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kotlin_extensions() {
        assert!(is_kotlin_extension(b"main.kt"));
        assert!(is_kotlin_extension(b"build.gradle.kts"));
        assert!(!is_kotlin_extension(b"main.k"));
    }

    #[test]
    fn test_kotlin_keyword_table_and_mph() {
        assert_eq!(KEYWORD_NAMES.len(), KEYWORD_COUNT);
        assert_eq!(KEYWORD_LENS.len(), KEYWORD_COUNT);
        for (i, name) in KEYWORD_NAMES.iter().enumerate() {
            let b = name.as_bytes();
            let h = kw_hash(b[0], b[b.len() / 2], b[b.len() - 1], b.len());
            assert_eq!(KW_TABLE[h], i as u8);
        }
    }

    #[test]
    fn test_kotlin_comments_and_strings() {
        let code = b"// single comment\n/* nested /* block */ comment */\npackage com.example\nclass Foo {\n    val s = \"\"\"\n    multiline\n    string\n    \"\"\"\n    fun bar() {}\n}\n";
        let mut acc = ParseResult::<KEYWORD_COUNT>::default();
        parse_into(code, &mut acc);
        assert_eq!(acc.comment_lines, 2);
        assert_eq!(acc.code_lines, 8);
        assert_eq!(acc.lines_processed, 10);
        assert_eq!(acc.total_keywords, 4);
    }
}
