use super::{
    finish_trailing_line, STATE_CODE, STATE_COMMENT, compute_keyword_lens, compute_kw_max_len, compute_kw_min_len, count_newlines_avx2,
    find_byte_avx2, kw_match_2_to_16, make_token_lut_lo, skip_ident_cont_avx2, skip_literal,
    skip_nested_block_comment, skip_non_tokens_with_lut_avx2, ParseResult, IDENT_CONT, TOK_CODE,
    TOK_NL, TOK_WS,
};

pub const KEYWORD_COUNT: usize = 55;
const KW_SLOT_MASK: usize = 255;
const KW_EMPTY: u8 = 255;

pub const KEYWORD_NAMES: [&str; KEYWORD_COUNT] = [
    "Self",
    "abstract",
    "as",
    "async",
    "await",
    "become",
    "box",
    "break",
    "const",
    "continue",
    "crate",
    "do",
    "dyn",
    "else",
    "enum",
    "extern",
    "false",
    "final",
    "fn",
    "for",
    "gen",
    "if",
    "impl",
    "in",
    "let",
    "loop",
    "macro",
    "macro_rules",
    "match",
    "mod",
    "move",
    "mut",
    "override",
    "priv",
    "pub",
    "raw",
    "ref",
    "return",
    "self",
    "static",
    "struct",
    "super",
    "trait",
    "true",
    "try",
    "type",
    "typeof",
    "union",
    "unsafe",
    "unsized",
    "use",
    "virtual",
    "where",
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
    let v = (first as usize)
        .wrapping_mul(183)
        .wrapping_add((mid as usize).wrapping_mul(115))
        .wrapping_add((last as usize).wrapping_mul(211))
        .wrapping_add(len.wrapping_mul(155));
    (v ^ (v >> 6)) & KW_SLOT_MASK
}

const KW_TABLE: [u8; 256] = {
    let mut table = [KW_EMPTY; 256];
    let mut i = 0;
    while i < KEYWORD_COUNT {
        let name = KEYWORD_NAMES[i].as_bytes();
        let len = KEYWORD_LENS[i] as usize;
        assert!(name.len() == len);
        assert!(len >= KW_MIN_LEN && len <= KW_MAX_LEN);
        let h = kw_hash(name[0], name[len >> 1], name[len - 1], len);
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

#[inline(always)]
fn skip_non_tokens_avx2(ptr: &[u8], i: usize) -> usize {
    skip_non_tokens_with_lut_avx2(ptr, i, &TOKEN_KIND, &TOKEN_LUT_LO)
}

#[inline(always)]
fn skip_rust_raw_string(content: &[u8], start_i: usize, hashes: usize) -> usize {
    let content_len = content.len();
    let mut i = start_i;
    while i < content_len {
        let Some(pos) = find_byte_avx2(&content[i..], b'"') else {
            return content_len;
        };
        let quote_idx = i + pos;
        let after_quote = quote_idx + 1;
        if after_quote + hashes > content_len {
            return content_len;
        }
        let mut matched = true;
        for k in 0..hashes {
            if content[after_quote + k] != b'#' {
                matched = false;
                break;
            }
        }
        if matched {
            return after_quote + hashes;
        }
        i = after_quote;
    }
    content_len
}

#[cfg(test)]
#[inline(always)]
pub fn is_rust_extension(name: &[u8]) -> bool {
    super::detect_language(name, super::LANG_ALL_MASK) == Some(super::LangId::Rust)
}

pub fn parse_into(content: &[u8], acc: &mut ParseResult<KEYWORD_COUNT>) {
    let content_len = content.len();
    if content_len == 0 {
        return;
    }

    let mut line_state = 0u8;
    let mut i = 0usize;
    while i < content_len {
        if line_state == 0 {
            while i < content_len && (content[i] == b' ' || content[i] == b'\t' || content[i] == b'\r') {
                i += 1;
            }
            if i >= content_len {
                break;
            }
            if content[i] == b'\n' {
                acc.blank_lines += 1;
                acc.lines_processed += 1;
                i += 1;
                continue;
            }
        }

        if TOKEN_KIND[content[i] as usize] <= TOK_CODE && content[i] != b'\n' {
            line_state |= STATE_CODE;
            i += 1;
            if i < content_len && TOKEN_KIND[content[i] as usize] <= TOK_CODE && content[i] != b'\n' {
                i = skip_non_tokens_avx2(content, i);
            }
            if i >= content_len {
                break;
            }
        }

        match TOKEN_KIND[content[i] as usize] {
            TOK_NL => {
                if (line_state & STATE_CODE) != 0 {
                    acc.code_lines += 1;
                } else if (line_state & STATE_COMMENT) != 0 {
                    acc.comment_lines += 1;
                } else {
                    acc.blank_lines += 1;
                }
                acc.lines_processed += 1;
                line_state = 0;
                i += 1;
            }
            TOK_IDENT => {
                line_state |= STATE_CODE;
                let start = i;
                i = skip_ident_cont_avx2(content, i + 1);
                debug_assert!(i > start && i <= content_len);
                let n = i - start;
                let cand = &content[start..i];

                if i < content_len {
                    let next_b = content[i];
                    if next_b == b'#'
                        && (cand == b"r" || cand == b"br" || cand == b"cr")
                    {
                        let mut hash_end = i + 1;
                        while hash_end < content_len && content[hash_end] == b'#' {
                            hash_end += 1;
                        }
                        let hashes = hash_end - i;
                        if hashes == 1
                            && cand == b"r"
                            && hash_end < content_len
                            && IDENT_CONT[content[hash_end] as usize] != 0
                        {
                            i = skip_ident_cont_avx2(content, hash_end + 1);
                            continue;
                        }
                        if hash_end < content_len && content[hash_end] == b'"' {
                            let next_i = skip_rust_raw_string(content, hash_end + 1, hashes);
                            let nl = count_newlines_avx2(&content[hash_end + 1..next_i]);
                            if nl > 0 {
                                acc.code_lines += nl as i64;
                                acc.lines_processed += nl as i64;
                            }
                            i = next_i;
                            continue;
                        }
                    } else if next_b == b'"'
                        && (cand == b"r" || cand == b"br" || cand == b"cr")
                    {
                        let next_i = skip_rust_raw_string(content, i + 1, 0);
                        let nl = count_newlines_avx2(&content[i + 1..next_i]);
                        if nl > 0 {
                            acc.code_lines += nl as i64;
                            acc.lines_processed += nl as i64;
                        }
                        i = next_i;
                        continue;
                    }
                }

                if n.wrapping_sub(KW_MIN_LEN) <= KW_MAX_LEN - KW_MIN_LEN {
                    let h = kw_hash(content[start], content[start + (n >> 1)], content[i - 1], n);
                    let id = KW_TABLE[h];
                    if id != KW_EMPTY {
                        let idx = id as usize;
                        debug_assert!(idx < KEYWORD_COUNT);
                        if KEYWORD_LENS[idx] as usize == n
                            && kw_match_2_to_16(cand, KEYWORD_NAMES[idx].as_bytes())
                        {
                            acc.keyword_counts[idx] += 1;
                            acc.total_keywords += 1;
                        }
                    }
                }
            }
            TOK_SLASH => {
                if i + 1 < content_len {
                    let next_b = content[i + 1];
                    if next_b == b'/' {
                        line_state |= STATE_COMMENT;
                        let Some(pos) = find_byte_avx2(&content[i + 2..], b'\n') else {
                            break;
                        };
                        let nl_idx = i + 2 + pos;
                        if (line_state & STATE_CODE) != 0 {
                            acc.code_lines += 1;
                        } else if (line_state & STATE_COMMENT) != 0 {
                            acc.comment_lines += 1;
                        } else {
                            acc.blank_lines += 1;
                        }
                        acc.lines_processed += 1;
                        line_state = 0;
                        i = nl_idx + 1;
                        continue;
                    } else if next_b == b'*' {
                        line_state |= STATE_COMMENT;
                        let end_i = skip_nested_block_comment(content, i + 2);
                        if let Some(first_nl) = find_byte_avx2(&content[i + 2..end_i], b'\n') {
                            if (line_state & STATE_CODE) != 0 {
                                acc.code_lines += 1;
                            } else {
                                acc.comment_lines += 1;
                            }
                            acc.lines_processed += 1;
                            let first_nl_abs = i + 2 + first_nl;
                            let inter_nls = count_newlines_avx2(&content[first_nl_abs + 1..end_i]);
                            acc.comment_lines += inter_nls as i64;
                            acc.lines_processed += inter_nls as i64;
                            line_state = STATE_COMMENT;
                        }
                        i = end_i;
                        continue;
                    }
                }
                line_state |= STATE_CODE;
                i += 1;
            }
            TOK_QUOTE => {
                line_state |= STATE_CODE;
                let end_i = skip_literal(content, i + 1, b'"');
                let nl = count_newlines_avx2(&content[i + 1..end_i]);
                if nl > 0 {
                    acc.code_lines += nl as i64;
                    acc.lines_processed += nl as i64;
                }
                i = end_i;
            }
            TOK_APOSTROPHE => {
                line_state |= STATE_CODE;
                if i + 1 < content_len {
                    let next_b = content[i + 1];
                    if next_b == b'\\' {
                        let end_i = skip_literal(content, i + 1, b'\'');
                        let nl = count_newlines_avx2(&content[i + 1..end_i]);
                        if nl > 0 {
                            acc.code_lines += nl as i64;
                            acc.lines_processed += nl as i64;
                        }
                        i = end_i;
                        continue;
                    }
                    if i + 2 < content_len && content[i + 2] == b'\'' {
                        i += 3;
                        continue;
                    }
                    if IDENT_CONT[next_b as usize] != 0 {
                        i += 1;
                        continue;
                    }
                }
                i += 1;
            }
            _ => {
                line_state |= STATE_CODE;
                i += 1;
            }
        }
    }
    finish_trailing_line(content, line_state, acc);
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn find_kw_id(name: &str) -> usize {
        KEYWORD_NAMES.iter().position(|&kw| kw == name).unwrap()
    }

    #[test]
    fn test_rust_keyword_table_and_mph() {
        assert_eq!(KEYWORD_COUNT, 55);
        let mut seen_slot = [false; 256];
        for i in 0..KEYWORD_COUNT {
            let kw = KEYWORD_NAMES[i];
            let len = kw.len();
            assert_eq!(len, KEYWORD_LENS[i] as usize);
            assert!((2..=KW_MAX_LEN).contains(&len));

            let bytes = kw.as_bytes();
            let h = kw_hash(bytes[0], bytes[len >> 1], bytes[len - 1], len);
            assert!(!seen_slot[h]);
            seen_slot[h] = true;
            assert_eq!(KW_TABLE[h], i as u8);
            assert!(kw_match_2_to_16(bytes, bytes));

            let mut res = ParseResult::default();
            parse_into(bytes, &mut res);
            assert_eq!(res.total_keywords, 1, "Failed to match keyword: {}", kw);
            assert_eq!(res.keyword_counts[i], 1);
            assert_eq!(res.lines_processed, 1);
        }

        let non_keywords = [
            "a",
            "i",
            "println",
            "vec",
            "format",
            "todo",
            "unimplemented",
            "panic",
            "assert",
            "assert_eq",
            "debug_assert",
            "Option",
            "Some",
            "None",
            "Result",
            "Ok",
            "Err",
            "Box",
            "Vec",
            "String",
            "u8",
            "u16",
            "u32",
            "u64",
            "usize",
            "i8",
            "i16",
            "i32",
            "i64",
            "isize",
            "f32",
            "f64",
            "bool",
            "char",
            "str",
            "int",
            "void",
            "class",
            "namespace",
            "template",
            "typename",
            "constexpr",
            "nullptr",
            "switch",
            "case",
            "default",
            "goto",
            "inline",
            "volatile",
            "register",
            "sizeof",
            "typedef",
        ];
        for s in non_keywords {
            let mut res = ParseResult::default();
            parse_into(s.as_bytes(), &mut res);
            assert_eq!(res.total_keywords, 0, "False positive on: {}", s);
        }

        for b in 0..=255u8 {
            let mut block = [b' '; 64];
            block[19] = b;
            let pos = skip_non_tokens_avx2(&block, 0);
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
    fn test_rust_extensions() {
        assert!(is_rust_extension(b"main.rs"));
        assert!(is_rust_extension(b"lib.RS"));
        assert!(is_rust_extension(b"src/lang/rust.Rs"));

        assert!(!is_rust_extension(b"main.c"));
        assert!(!is_rust_extension(b"main.cpp"));
        assert!(!is_rust_extension(b"main.r"));
        assert!(!is_rust_extension(b"rs"));
        assert!(!is_rust_extension(b".rs"));
        assert!(!is_rust_extension(b""));
    }

    #[test]
    fn test_rust_lifetimes_and_char_literals() {
        let src = concat!(
            "pub fn borrow<'a, 'b: 'a>(x: &'a str, y: &'static str) -> &'a str {\n",
            "    let c1 = '\\'';\n",
            "    let c2 = '\"';\n",
            "    let c3 = '\\\\';\n",
            "    let c4 = 'x';\n",
            "    let b1 = b'\\'';\n",
            "    'outer: loop {\n",
            "        if true { break 'outer; }\n",
            "    }\n",
            "    return x;\n",
            "}\n"
        );

        let mut res = ParseResult::default();
        parse_into(src.as_bytes(), &mut res);

        assert_eq!(res.keyword_counts[find_kw_id("pub")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("fn")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("static")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("let")], 5);
        assert_eq!(res.keyword_counts[find_kw_id("loop")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("if")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("true")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("break")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("return")], 1);
        assert_eq!(res.total_keywords, 13);
        assert_eq!(res.lines_processed, 11);
    }

    #[test]
    fn test_rust_raw_strings_and_byte_strings() {
        let src = concat!(
            "let s1 = \"fn let mut pub return struct enum impl\";\n",
            "let s2 = \"escaped \\\"fn let return\\\" still in string\";\n",
            "let s3 = r\"raw string with \\\\ \\n and fn let mut return\";\n",
            "let s4 = r#\"raw string with \"quotes\" and fn let match return\"#;\n",
            "let s5 = r###\"raw string with \"# and \"## and async await unsafe\"###;\n",
            "let s6 = br#\"byte raw string with fn impl trait where\"#;\n",
            "let s7 = cr#\"c raw string with loop while for in\"#;\n",
            "pub fn done() -> bool { true }\n"
        );

        let mut res = ParseResult::default();
        parse_into(src.as_bytes(), &mut res);

        assert_eq!(res.keyword_counts[find_kw_id("let")], 7);
        assert_eq!(res.keyword_counts[find_kw_id("pub")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("fn")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("true")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("mut")], 0);
        assert_eq!(res.keyword_counts[find_kw_id("return")], 0);
        assert_eq!(res.keyword_counts[find_kw_id("match")], 0);
        assert_eq!(res.keyword_counts[find_kw_id("unsafe")], 0);
        assert_eq!(res.total_keywords, 10);
        assert_eq!(res.lines_processed, 8);
    }

    #[test]
    fn test_rust_nested_block_comments() {
        let src = concat!(
            "// line comment with fn let mut pub\n",
            "const X: usize = 1;\n",
            "/* outer block comment\n",
            "   fn let mut\n",
            "   /* nested inner block comment\n",
            "      struct enum impl trait\n",
            "   */\n",
            "   still inside outer block comment: return match if elseloop\n",
            "*/\n",
            "pub struct Foo;\n",
            "/* unclosed nested /* comment at EOF: fn let pub return"
        );

        let mut res = ParseResult::default();
        parse_into(src.as_bytes(), &mut res);

        assert_eq!(res.keyword_counts[find_kw_id("const")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("pub")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("struct")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("fn")], 0);
        assert_eq!(res.keyword_counts[find_kw_id("let")], 0);
        assert_eq!(res.keyword_counts[find_kw_id("return")], 0);
        assert_eq!(res.total_keywords, 3);
        assert_eq!(res.lines_processed, 11);
    }

    #[test]
    fn test_rust_raw_identifiers_and_macros() {
        let src = concat!(
            "macro_rules! my_macro {\n",
            "    () => {\n",
            "        let r#type = 1;\n",
            "        let r#match = 2;\n",
            "        let r#fn = 3;\n",
            "        let r#async = 4;\n",
            "        let normal = r#let;\n",
            "        println!(\"fn let mut\");\n",
            "    };\n",
            "}\n",
            "impl Self {\n",
            "    pub unsafe fn check(&self) -> bool {\n",
            "        let ptr = &raw const self;\n",
            "        false\n",
            "    }\n",
            "}\n"
        );

        let mut res = ParseResult::default();
        parse_into(src.as_bytes(), &mut res);

        assert_eq!(res.keyword_counts[find_kw_id("macro_rules")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("let")], 6);
        assert_eq!(res.keyword_counts[find_kw_id("impl")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("Self")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("pub")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("unsafe")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("fn")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("self")], 2);
        assert_eq!(res.keyword_counts[find_kw_id("raw")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("const")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("false")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("type")], 0);
        assert_eq!(res.keyword_counts[find_kw_id("match")], 0);
        assert_eq!(res.keyword_counts[find_kw_id("async")], 0);
        assert_eq!(res.total_keywords, 17);
    }

    #[test]
    fn test_rust_avx2_boundaries_and_eof_edge_cases() {
        for start_off in 0..96 {
            for &kw_str in &[
                "as",
                "fn",
                "in",
                "let",
                "Self",
                "self",
                "match",
                "return",
                "unsafe",
                "continue",
                "macro_rules",
            ] {
                let mut b = vec![b' '; 160];
                let kw_bytes = kw_str.as_bytes();
                b[start_off..start_off + kw_bytes.len()].copy_from_slice(kw_bytes);
                let mut r = ParseResult::default();
                parse_into(&b, &mut r);
                assert_eq!(r.total_keywords, 1);
                assert_eq!(r.keyword_counts[find_kw_id(kw_str)], 1);
            }
        }

        for empty_or_edge in [
            "",
            "'",
            "'a",
            "''",
            "'\\'",
            "r\"",
            "r#\"",
            "r#",
            "r#type",
            "/*",
            "/* /* */",
            "//",
            "1_000_000",
        ] {
            let mut res = ParseResult::default();
            parse_into(empty_or_edge.as_bytes(), &mut res);
            assert_eq!(
                res.total_keywords, 0,
                "Unexpected match on: {}",
                empty_or_edge
            );
        }
    }

    #[test]
    fn test_rust_lines_code_comments_blanks() {
        let src = concat!(
            "// rust single comment\n",
            "\n",
            "fn main() {\n",
            "    /* nested\n",
            "       /* comment */\n",
            "       block */\n",
            "    let s = r#\"\n",
            "multi raw\n",
            "\"#;\n",
            "}\n"
        );
        let mut res = ParseResult::default();
        parse_into(src.as_bytes(), &mut res);
        assert_eq!(res.lines_processed, 10);
        assert_eq!(res.blank_lines, 1);
        assert_eq!(res.comment_lines, 4);
        assert_eq!(res.code_lines, 5);
    }
}
