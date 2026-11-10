use super::{
    compute_keyword_lens, compute_kw_max_len, compute_kw_min_len, count_newlines_avx2,
    find_byte_avx2, find_star_slash, kw_match_2_to_16, make_token_lut_lo, skip_literal,
    unified_parse_into, LangLexer, ParseResult, IDENT_CONT, STATE_CODE, STATE_COMMENT, TOK_CODE,
    TOK_NL, TOK_WS,
};

#[cfg(test)]
use super::{skip_ident_cont_avx2, skip_non_tokens_with_lut_avx2};

pub const KEYWORD_COUNT: usize = 59;
const KW_SLOT_MASK: usize = 255;
const KW_EMPTY: u8 = 255;

pub const KEYWORD_NAMES: [&str; KEYWORD_COUNT] = [
    "auto",
    "break",
    "case",
    "char",
    "const",
    "continue",
    "default",
    "do",
    "double",
    "else",
    "enum",
    "extern",
    "float",
    "for",
    "goto",
    "if",
    "int",
    "long",
    "register",
    "return",
    "short",
    "signed",
    "sizeof",
    "static",
    "struct",
    "switch",
    "typedef",
    "union",
    "unsigned",
    "void",
    "volatile",
    "while",
    "inline",
    "restrict",
    "_Bool",
    "_Complex",
    "_Imaginary",
    "_Alignas",
    "_Alignof",
    "_Atomic",
    "_Static_assert",
    "_Noreturn",
    "_Thread_local",
    "_Generic",
    "_BitInt",
    "typeof",
    "typeof_unqual",
    "_Decimal128",
    "_Decimal32",
    "_Decimal64",
    "alignas",
    "alignof",
    "bool",
    "constexpr",
    "false",
    "nullptr",
    "static_assert",
    "thread_local",
    "true",
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
const fn kw_hash(first: u8, last: u8, len: usize) -> usize {
    ((first as usize)
        .wrapping_add((last as usize).wrapping_mul(9))
        .wrapping_add(len.wrapping_mul(11)))
        & KW_SLOT_MASK
}

const KW_TABLE: [u8; 256] = {
    let mut table = [KW_EMPTY; 256];
    let mut i = 0;
    while i < KEYWORD_COUNT {
        let name = KEYWORD_NAMES[i].as_bytes();
        let len = KEYWORD_LENS[i] as usize;
        assert!(name.len() == len);
        assert!(len >= KW_MIN_LEN && len <= KW_MAX_LEN);
        let h = kw_hash(name[0], name[len - 1], len);
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
pub fn is_c_extension(name: &[u8]) -> bool {
    super::detect_language(name, super::LANG_ALL_MASK) == Some(super::LangId::C)
}

pub struct CLexer;

impl LangLexer<KEYWORD_COUNT> for CLexer {
    const TOKEN_KIND: [u8; 256] = TOKEN_KIND;
    const TOKEN_LUT_LO: [i8; 32] = TOKEN_LUT_LO;
    const HAS_DOLLAR_IDENT: bool = false;

    #[inline(always)]
    fn match_keyword(cand: &[u8]) -> Option<usize> {
        let n = cand.len();
        if n.wrapping_sub(KW_MIN_LEN) <= KW_MAX_LEN - KW_MIN_LEN {
            let h = kw_hash(cand[0], cand[n - 1], n);
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
                        let mut j = i + 2;
                        while j < content_len {
                            let Some(pos) = find_byte_avx2(&content[j..], b'\n') else {
                                j = content_len;
                                break;
                            };
                            let nl_idx = j + pos;
                            let mut check = nl_idx;
                            if check > j && content[check - 1] == b'\r' {
                                check -= 1;
                            }
                            let is_spliced = check > j && content[check - 1] == b'\\';
                            if (*line_state & STATE_CODE) != 0 {
                                acc.code_lines += 1;
                            } else if (*line_state & STATE_COMMENT) != 0 {
                                acc.comment_lines += 1;
                            } else {
                                acc.blank_lines += 1;
                            }
                            acc.lines_processed += 1;
                            if is_spliced {
                                *line_state = STATE_COMMENT;
                                j = nl_idx + 1;
                            } else {
                                *line_state = 0;
                                j = nl_idx + 1;
                                break;
                            }
                        }
                        return j;
                    } else if next_b == b'*' {
                        *line_state |= STATE_COMMENT;
                        if let Some(pos) = find_star_slash(&content[i + 2..]) {
                            let star_slash_pos = i + 2 + pos;
                            if let Some(first_nl) = find_byte_avx2(&content[i + 2..star_slash_pos], b'\n') {
                                if (*line_state & STATE_CODE) != 0 {
                                    acc.code_lines += 1;
                                } else {
                                    acc.comment_lines += 1;
                                }
                                acc.lines_processed += 1;
                                let first_nl_abs = i + 2 + first_nl;
                                let inter_nls = count_newlines_avx2(&content[first_nl_abs + 1..star_slash_pos]);
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
                let end_i = skip_literal(content, i + 1, b'"');
                let nl = count_newlines_avx2(&content[i + 1..end_i]);
                if nl > 0 {
                    acc.code_lines += nl as i64;
                    acc.lines_processed += nl as i64;
                }
                end_i
            }
            TOK_APOSTROPHE => {
                if i > 0 && i + 1 < content_len {
                    let prev = content[i - 1];
                    let next = content[i + 1];
                    if IDENT_CONT[prev as usize] != 0 && IDENT_CONT[next as usize] != 0 {
                        *line_state |= STATE_CODE;
                        return i + 1;
                    }
                }
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
    unified_parse_into::<KEYWORD_COUNT, CLexer>(content, acc);
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn find_kw_id(name: &str) -> usize {
        KEYWORD_NAMES.iter().position(|&kw| kw == name).unwrap()
    }

    #[test]
    fn test_keyword_table_and_mph() {
        assert_eq!(KEYWORD_COUNT, 59);
        let mut seen_slot = [false; 256];
        for i in 0..KEYWORD_COUNT {
            let kw = KEYWORD_NAMES[i];
            let len = kw.len();
            assert_eq!(len, KEYWORD_LENS[i] as usize);
            assert!((2..=KW_MAX_LEN).contains(&len));

            let bytes = kw.as_bytes();
            let h = kw_hash(bytes[0], bytes[len - 1], len);
            assert!(!seen_slot[h]);
            seen_slot[h] = true;
            assert_eq!(KW_TABLE[h], i as u8);
            assert!(kw_match_2_to_16(bytes, bytes));

            let mut res = ParseResult::default();
            parse_into(bytes, &mut res);
            assert_eq!(res.total_keywords, 1);
            assert_eq!(res.keyword_counts[i], 1);
            assert_eq!(res.lines_processed, 1);
        }

        let non_keywords = [
            "a",
            "i",
            "if_",
            "_if",
            "iff",
            "inta",
            "int_t",
            "returning",
            "autoo",
            "breaks",
            "caser",
            "chars",
            "consts",
            "continues",
            "defaults",
            "d0",
            "doubles",
            "elses",
            "enums",
            "externs",
            "floats",
            "forr",
            "gotos",
            "longs",
            "registers",
            "shorts",
            "signeds",
            "sizeofs",
            "statics",
            "structs",
            "switchs",
            "typedefs",
            "unions",
            "unsigneds",
            "voids",
            "volatiles",
            "whiles",
            "inlines",
            "restricts",
            "_Bools",
            "boolean",
            "nullptrs",
            "constexprs",
            "thread_locals",
            "static_asserts",
            "_Static_asserts",
            "class",
            "namespace",
            "template",
            "typename",
            "virtual",
            "override",
            "final",
            "private",
            "protected",
            "public",
            "friend",
            "using",
            "new",
            "delete",
            "try",
            "catch",
            "throw",
            "noexcept",
            "mutable",
            "explicit",
            "operator",
            "dynamic_cast",
            "static_cast",
            "reinterpret_cast",
            "const_cast",
            "typeid",
            "concept",
            "requires",
            "co_await",
            "co_yield",
            "co_return",
        ];
        for s in non_keywords {
            let mut res = ParseResult::default();
            parse_into(s.as_bytes(), &mut res);
            assert_eq!(res.total_keywords, 0);
        }

        for b in 0..=255u8 {
            let mut block = [b' '; 64];
            block[19] = b;
            let pos = skip_non_tokens_with_lut_avx2(&block, 0, &TOKEN_KIND, &TOKEN_LUT_LO);
            if TOKEN_KIND[b as usize] > TOK_CODE || b == b'\n' {
                assert_eq!(pos, 19);
            } else {
                assert_eq!(pos, block.len());
            }

            let mut block = [b'a'; 64];
            block[19] = b;
            let id_pos = skip_ident_cont_avx2(&block, 0);
            if IDENT_CONT[b as usize] == 0 {
                assert_eq!(id_pos, 19);
            } else {
                assert_eq!(id_pos, block.len());
            }
        }
    }

    #[test]
    fn test_extensions() {
        assert!(is_c_extension(b"main.c"));
        assert!(is_c_extension(b"header.h"));
        assert!(is_c_extension(b"UPPER.C"));
        assert!(is_c_extension(b"UPPER.H"));
        assert!(is_c_extension(b"a/b/c/foo.bar.c"));

        assert!(!is_c_extension(b"main.cpp"));
        assert!(!is_c_extension(b"main.cc"));
        assert!(!is_c_extension(b"main.cxx"));
        assert!(!is_c_extension(b"main.c++"));
        assert!(!is_c_extension(b"header.hpp"));
        assert!(!is_c_extension(b"header.hxx"));
        assert!(!is_c_extension(b"header.h++"));
        assert!(!is_c_extension(b" header.hh"));
        assert!(!is_c_extension(b"script.py"));
        assert!(!is_c_extension(b"Makefile"));
        assert!(!is_c_extension(b"c"));
        assert!(!is_c_extension(b""));
    }

    #[test]
    fn test_comments_and_splicing() {
        let src = concat!(
            "// single line comment with if int return while\n",
            "int a = 1;\n",
            "// spliced line comment \\\n",
            "if (a) return 0;\n",
            "// multi-spliced line comment \\\r\n",
            "while (1) \\\n",
            "for (;;)\n",
            "return a;\n",
            "/* block comment with\n",
            "   struct union enum typedef static const\n",
            "   * / ** / ***/\n",
            "char b = 2;\n",
            "/* unclosed block comment at EOF: goto switch case default"
        );

        let mut res = ParseResult::default();
        parse_into(src.as_bytes(), &mut res);

        assert_eq!(res.total_keywords, 3);
        assert_eq!(res.keyword_counts[find_kw_id("int")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("return")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("char")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("if")], 0);
        assert_eq!(res.keyword_counts[find_kw_id("while")], 0);
        assert_eq!(res.keyword_counts[find_kw_id("for")], 0);
        assert_eq!(res.keyword_counts[find_kw_id("goto")], 0);
        assert_eq!(res.lines_processed, 13);
    }

    #[test]
    fn test_literals_and_digit_separators() {
        let src = concat!(
            "const char *s1 = \"if while for return struct\";\n",
            "const char *s2 = \"escaped \\\"if return\\\" still in string\";\n",
            "const char *s3 = \"escaped backslash at end \\\\\"; if (1) {}\n",
            "char c1 = '\"'; if (1) {}\n",
            "char c2 = '\\''; while (0) {}\n",
            "char c3 = '\\\\'; for (;;) {}\n",
            "int x = 1'000'000; return x;\n",
            "unsigned y = 0xDEAD'BEEF'CAFEu; break;\n",
            "int z = 1'2'3; if (z) continue;\n"
        );

        let mut res = ParseResult::default();
        parse_into(src.as_bytes(), &mut res);

        assert_eq!(res.keyword_counts[find_kw_id("const")], 3);
        assert_eq!(res.keyword_counts[find_kw_id("char")], 6);
        assert_eq!(res.keyword_counts[find_kw_id("if")], 3);
        assert_eq!(res.keyword_counts[find_kw_id("while")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("for")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("int")], 2);
        assert_eq!(res.keyword_counts[find_kw_id("return")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("unsigned")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("break")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("continue")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("struct")], 0);
        assert_eq!(res.total_keywords, 20);
    }

    #[test]
    fn test_avx2_boundaries() {
        let len = 32 * 300 + 17;
        let mut buf = vec![b' '; len];
        let mut expected_nl = 0usize;
        for i in (0..len).step_by(7) {
            buf[i] = b'\n';
            expected_nl += 1;
        }
        assert_eq!(count_newlines_avx2(&buf), expected_nl);

        buf.fill(b' ');
        let kw = b"_Static_assert";
        let kw_len = kw.len();
        let mut placed = 0i64;
        let mut off = 25usize;
        while off + kw_len < len {
            buf[off..off + kw_len].copy_from_slice(kw);
            placed += 1;
            off += 64;
        }

        let mut res = ParseResult::default();
        parse_into(&buf, &mut res);
        assert_eq!(res.total_keywords, placed);
        assert_eq!(res.keyword_counts[find_kw_id("_Static_assert")], placed);

        for start_off in 0..96 {
            for &kw_str in &["do", "if", "int", "return", "_Static_assert"] {
                let mut b = vec![b' '; 160];
                let kw_bytes = kw_str.as_bytes();
                b[start_off..start_off + kw_bytes.len()].copy_from_slice(kw_bytes);
                let mut r = ParseResult::default();
                parse_into(&b, &mut r);
                assert_eq!(r.total_keywords, 1);
                assert_eq!(r.keyword_counts[find_kw_id(kw_str)], 1);
            }
        }
    }

    #[test]
    fn test_malformed_and_eof_edge_cases() {
        for empty_or_tiny in [
            "",
            "/",
            "*",
            "\"",
            "'",
            "\\",
            "_",
            "a",
            "\n",
            "//",
            "/*",
            "*/",
            "/a",
            "''",
            "\"\"",
            "\"unclosed string if return",
            "\"unclosed string with escape \\",
            "'unclosed char if return",
            "'\\",
        ] {
            let mut res = ParseResult::default();
            parse_into(empty_or_tiny.as_bytes(), &mut res);
            assert_eq!(res.total_keywords, 0);
        }

        let tricky = concat!(
            "'a' if /**/ return /*/*/ while ///*\n",
            "for /* // */ int x = a / b / c; '"
        );
        let mut res = ParseResult::default();
        parse_into(tricky.as_bytes(), &mut res);
        assert_eq!(res.total_keywords, 5);
        assert_eq!(res.keyword_counts[find_kw_id("if")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("return")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("while")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("for")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("int")], 1);
        assert_eq!(res.lines_processed, 2);
    }

    #[test]
    fn test_c_lines_code_comments_blanks() {
        let src = concat!(
            "// comment only\n",
            "\n",
            "   \t  \r\n",
            "int a = 1; // code and comment\n",
            "/* block comment\n",
            "   span line 2\n",
            "   span line 3 */\n",
            "/* block */ int b = 2;\n",
            "int c = 3; /* inline block */\n",
            "return a + b + c;\n"
        );
        let mut res = ParseResult::default();
        parse_into(src.as_bytes(), &mut res);
        assert_eq!(res.lines_processed, 10);
        assert_eq!(res.blank_lines, 2);
        assert_eq!(res.comment_lines, 4);
        assert_eq!(res.code_lines, 4);
    }
}
