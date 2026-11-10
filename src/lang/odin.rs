use super::{
    compute_keyword_lens, compute_kw_max_len, compute_kw_min_len, count_newlines_avx2,
    find_byte_avx2, kw_match_2_to_16, make_token_lut_lo, skip_literal, skip_nested_block_comment,
    skip_triple_literal, unified_parse_into, LangLexer, ParseResult, STATE_CODE, STATE_COMMENT,
    STATE_NONE, TOK_CODE, TOK_NL, TOK_WS,
};

pub const KEYWORD_COUNT: usize = 39;
const KW_SLOT_MASK: usize = 255;
const KW_EMPTY: u8 = 255;

pub const KEYWORD_NAMES: [&str; KEYWORD_COUNT] = [
    "asm",
    "auto_cast",
    "bit_field",
    "bit_set",
    "break",
    "case",
    "cast",
    "context",
    "continue",
    "defer",
    "distinct",
    "do",
    "dynamic",
    "else",
    "enum",
    "fallthrough",
    "for",
    "foreign",
    "if",
    "import",
    "in",
    "map",
    "matrix",
    "not_in",
    "or_break",
    "or_continue",
    "or_else",
    "or_return",
    "package",
    "proc",
    "return",
    "struct",
    "switch",
    "transmute",
    "typeid",
    "union",
    "using",
    "when",
    "where",
];

pub const KEYWORD_LENS: [u8; KEYWORD_COUNT] = compute_keyword_lens(&KEYWORD_NAMES);
pub const KW_MIN_LEN: usize = compute_kw_min_len(&KEYWORD_NAMES);
pub const KW_MAX_LEN: usize = compute_kw_max_len(&KEYWORD_NAMES);

const _: () = assert!(KEYWORD_NAMES.len() == KEYWORD_COUNT);
const _: () = assert!(KEYWORD_LENS.len() == KEYWORD_COUNT);


#[inline(always)]
const fn kw_hash(first: u8, mid: u8, last: u8, len: usize) -> usize {
    let val = (first as usize)
        .wrapping_mul(1176)
        .wrapping_add((mid as usize).wrapping_mul(2141))
        .wrapping_add((last as usize).wrapping_mul(150))
        .wrapping_add(len.wrapping_mul(3013));
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

const TOK_IDENT: u8 = 3;
const TOK_SLASH: u8 = 4;
const TOK_QUOTE: u8 = 5;
const TOK_APOSTROPHE: u8 = 6;
const TOK_BACKTICK: u8 = 7;

pub const TOKEN_KIND: [u8; 256] = {
    let mut t = [TOK_CODE; 256];
    t[b' ' as usize] = TOK_WS;
    t[b'\t' as usize] = TOK_WS;
    t[b'\r' as usize] = TOK_WS;
    t[0x0b_usize] = TOK_WS;
    t[0x0c_usize] = TOK_WS;
    t[b'\n' as usize] = TOK_NL;

    let mut c = b'a';
    while c <= b'z' {
        t[c as usize] = TOK_IDENT;
        c += 1;
    }
    let mut c = b'A';
    while c <= b'Z' {
        t[c as usize] = TOK_IDENT;
        c += 1;
    }
    t[b'_' as usize] = TOK_IDENT;

    t[b'/' as usize] = TOK_SLASH;
    t[b'"' as usize] = TOK_QUOTE;
    t[b'\'' as usize] = TOK_APOSTROPHE;
    t[b'`' as usize] = TOK_BACKTICK;
    t
};

pub const TOKEN_LUT_LO: [i8; 32] = make_token_lut_lo(&TOKEN_KIND);

#[cfg(test)]
#[inline(always)]
pub fn is_odin_extension(name: &[u8]) -> bool {
    super::detect_language(name, super::LANG_ALL_MASK) == Some(super::LangId::Odin)
}

pub struct OdinLexer;

impl LangLexer<KEYWORD_COUNT> for OdinLexer {
    const TOKEN_KIND: [u8; 256] = TOKEN_KIND;
    const TOKEN_LUT_LO: [i8; 32] = TOKEN_LUT_LO;
    const HAS_DOLLAR_IDENT: bool = false;
    const SUPPORTS_TEMPLATES: bool = false;

    #[inline(always)]
    fn match_keyword(cand: &[u8]) -> Option<usize> {
        let n = cand.len();
        if n.wrapping_sub(KW_MIN_LEN) <= KW_MAX_LEN - KW_MIN_LEN {
            let h = kw_hash(cand[0], cand[n / 2], cand[n - 1], n);
            let slot = KW_TABLE[h];
            if slot != KW_EMPTY {
                let idx = slot as usize;
                if len_matches(n, idx)
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
                        let Some(pos) = find_byte_avx2(&content[i + 2..], b'\n') else {
                            return content_len;
                        };
                        let nl_idx = i + 2 + pos;
                        if (*line_state & STATE_CODE) != 0 {
                            acc.code_lines += 1;
                        } else if (*line_state & STATE_COMMENT) != 0 {
                            acc.comment_lines += 1;
                        } else {
                            acc.blank_lines += 1;
                        }
                        acc.lines_processed += 1;
                        *line_state = STATE_NONE;
                        return nl_idx + 1;
                    } else if next_b == b'*' {
                        *line_state |= STATE_COMMENT;
                        let end_i = skip_nested_block_comment(content, i + 2);
                        if let Some(first_nl) = find_byte_avx2(&content[i + 2..end_i], b'\n') {
                            if (*line_state & STATE_CODE) != 0 {
                                acc.code_lines += 1;
                            } else {
                                acc.comment_lines += 1;
                            }
                            acc.lines_processed += 1;
                            let first_nl_abs = i + 2 + first_nl;
                            let inter_nls = count_newlines_avx2(&content[first_nl_abs + 1..end_i]);
                            acc.comment_lines += inter_nls as i64;
                            acc.lines_processed += inter_nls as i64;
                            *line_state = STATE_COMMENT;
                        }
                        return end_i;
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
                    return end_i;
                }
                let end_i = skip_literal(content, i + 1, b'"');
                let nl = count_newlines_avx2(&content[i + 1..end_i]);
                if nl > 0 {
                    acc.code_lines += nl as i64;
                    acc.lines_processed += nl as i64;
                }
                end_i
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
            TOK_BACKTICK => {
                *line_state |= STATE_CODE;
                let end_i = match find_byte_avx2(&content[i + 1..], b'`') {
                    Some(pos) => i + 1 + pos + 1,
                    None => content_len,
                };
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
fn len_matches(len: usize, idx: usize) -> bool {
    len == KEYWORD_LENS[idx] as usize
}

#[inline(always)]
pub fn parse_into(content: &[u8], acc: &mut ParseResult<KEYWORD_COUNT>) {
    unified_parse_into::<KEYWORD_COUNT, OdinLexer>(content, acc);
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn find_kw_id(name: &str) -> usize {
        KEYWORD_NAMES.iter().position(|&kw| kw == name).unwrap()
    }

    #[test]
    fn test_odin_keyword_table_and_mph() {
        assert_eq!(KEYWORD_COUNT, 39);
        let mut seen = [false; 256];
        for i in 0..KEYWORD_COUNT {
            let kw = KEYWORD_NAMES[i];
            let len = kw.len();
            assert_eq!(len, KEYWORD_LENS[i] as usize);
            assert!((2..=KW_MAX_LEN).contains(&len));

            let bytes = kw.as_bytes();
            let first = bytes[0];
            let mid = bytes[len / 2];
            let last = bytes[len - 1];
            let h = kw_hash(first, mid, last, len);
            assert!(!seen[h]);
            seen[h] = true;
            assert_eq!(KW_TABLE[h], i as u8);

            let mut res = ParseResult::default();
            parse_into(bytes, &mut res);
            assert_eq!(res.total_keywords, 1);
            assert_eq!(res.keyword_counts[i], 1);
        }

        let non_keywords = ["main", "fmt", "println", "int", "string", "procs", "ifs"];
        for &nk in &non_keywords {
            let mut res = ParseResult::default();
            parse_into(nk.as_bytes(), &mut res);
            assert_eq!(res.total_keywords, 0);
        }
    }

    #[test]
    fn test_odin_extensions() {
        assert!(is_odin_extension(b"main.odin"));
        assert!(is_odin_extension(b"UPPER.ODIN"));
        assert!(is_odin_extension(b"path/to/module.odin"));
        assert!(!is_odin_extension(b"odin"));
        assert!(!is_odin_extension(b".odin"));
        assert!(!is_odin_extension(b"main.c"));
    }

    #[test]
    fn test_odin_comments_and_strings() {
        let src = b"package main\n\nimport \"core:fmt\"\n\n// line comment\n/* nested /* block */ comment */\nmain :: proc() {\n    s := \"hello // not comment\"\n    raw := `line 1\nline 2`\n    r := 'a'\n    return\n}\n";
        let mut res = ParseResult::default();
        parse_into(src, &mut res);

        assert_eq!(res.keyword_counts[find_kw_id("package")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("import")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("proc")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("return")], 1);
        assert_eq!(res.total_keywords, 4);
        assert_eq!(res.lines_processed, 13);
        assert_eq!(res.code_lines, 9);
        assert_eq!(res.comment_lines, 2);
        assert_eq!(res.blank_lines, 2);

        let mut res_mixed = ParseResult::default();
        parse_into(b"main :: proc() { /* nested /* \n */ end */\n}\n", &mut res_mixed);
        assert_eq!(res_mixed.lines_processed, 3);
        assert_eq!(res_mixed.code_lines, 2);
        assert_eq!(res_mixed.comment_lines, 1);
        assert_eq!(res_mixed.blank_lines, 0);
    }

    #[test]
    fn test_odin_triple_quotes() {
        let src = b"package main\n\nconst_s := \"\"\"line 1\nline 2 with proc and import\nline 3\"\"\"\n\nx := 1\n";
        let mut res = ParseResult::default();
        parse_into(src, &mut res);

        assert_eq!(res.keyword_counts[find_kw_id("package")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("proc")], 0);
        assert_eq!(res.keyword_counts[find_kw_id("import")], 0);
        assert_eq!(res.total_keywords, 1);
        assert_eq!(res.code_lines, 5);
        assert_eq!(res.comment_lines, 0);
        assert_eq!(res.blank_lines, 2);
    }

    #[test]
    fn test_odin_avx2_boundaries_and_eof() {
        for len in [31, 32, 33, 63, 64, 65, 127, 128, 129] {
            let mut buf = vec![b' '; len];
            buf.extend_from_slice(b"main :: proc() {\n    return\n}\n");
            let mut res = ParseResult::default();
            parse_into(&buf, &mut res);
            assert_eq!(res.keyword_counts[find_kw_id("proc")], 1);
            assert_eq!(res.keyword_counts[find_kw_id("return")], 1);
            assert_eq!(res.total_keywords, 2);
        }

        let mut res = ParseResult::default();
        parse_into(b"/* unclosed nested /* comment", &mut res);
        assert_eq!(res.total_keywords, 0);

        let mut res2 = ParseResult::default();
        parse_into(b"\"\"\"unclosed triple quote", &mut res2);
        assert_eq!(res2.total_keywords, 0);

        let mut res3 = ParseResult::default();
        parse_into(b"// unclosed line comment at eof", &mut res3);
        assert_eq!(res3.total_keywords, 0);
        assert_eq!(res3.comment_lines, 1);
        assert_eq!(res3.lines_processed, 1);
    }
}
