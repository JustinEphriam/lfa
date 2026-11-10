use super::{
    count_newlines_avx2, find_byte_avx2, kw_match_2_to_16, skip_ident_cont_avx2, skip_literal,
    skip_nested_block_comment, skip_triple_literal, ParseResult,
};

pub const KEYWORD_COUNT: usize = 39;
const KW_SLOT_MASK: usize = 255;
const KW_MAX_LEN: usize = 11;
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

pub const KEYWORD_LENS: [u8; KEYWORD_COUNT] = [
    3, 9, 9, 7, 5, 4, 4, 7, 8, 5, 8, 2, 7, 4, 4, 11, 3, 7, 2, 6, 2, 3, 6, 6, 8, 11, 7, 9, 7, 4,
    6, 6, 6, 9, 6, 5, 5, 4, 5,
];

const _: () = assert!(KEYWORD_NAMES.len() == KEYWORD_COUNT);
const _: () = assert!(KEYWORD_LENS.len() == KEYWORD_COUNT);

const TOK_WS: u8 = 0;
const TOK_NL: u8 = 1;
const TOK_CODE: u8 = 2;
const TOK_IDENT: u8 = 3;
const TOK_SLASH: u8 = 4;
const TOK_QUOTE: u8 = 5;
const TOK_APOSTROPHE: u8 = 6;
const TOK_BACKTICK: u8 = 7;

pub const TOKEN_KIND: [u8; 256] = [
    TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE,
    TOK_CODE, TOK_WS, TOK_NL, TOK_CODE, TOK_CODE, TOK_WS, TOK_CODE, TOK_CODE,
    TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE,
    TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE,
    TOK_WS, TOK_CODE, TOK_QUOTE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_APOSTROPHE,
    TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_SLASH,
    TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT,
    TOK_IDENT, TOK_IDENT, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE,
    TOK_CODE, TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT,
    TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT,
    TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT,
    TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_IDENT,
    TOK_BACKTICK, TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT,
    TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT,
    TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_IDENT,
    TOK_IDENT, TOK_IDENT, TOK_IDENT, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE,
    TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE,
    TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE,
    TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE,
    TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE,
    TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE,
    TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE,
    TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE,
    TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE,
    TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE,
    TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE,
    TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE,
    TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE,
    TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE,
    TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE,
    TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE,
    TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE, TOK_CODE,
];

pub const KW_SLOT: [u8; 256] = [
    1, 13, 255, 255, 255, 255, 255, 32, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 17, 255, 255, 255, 255, 255, 9, 255, 255, 29, 255, 255,
    255, 255, 2, 33, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 24, 255, 255, 255, 255, 26,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 6, 255, 255, 255, 4, 255, 255, 255, 255, 255,
    255, 255, 8, 255, 11, 255, 255, 255, 255, 255, 20, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 10, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 16, 255, 12, 255, 255, 255, 27, 28, 3, 255, 255, 255, 255, 255, 255,
    255, 37, 255, 255, 255, 255, 21, 23, 22, 255, 255, 255, 255, 255, 255, 35,
    255, 255, 255, 255, 34, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 36, 5, 255, 255, 255, 255, 255, 255,
    255, 255, 25, 255, 255, 255, 255, 255, 255, 255, 0, 255, 255, 255, 255, 255,
    38, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 14, 255,
    15, 19, 30, 255, 7, 255, 255, 255, 255, 255, 255, 255, 255, 255, 18, 255,
    31, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
];

#[inline(always)]
pub fn kw_hash(first: u8, mid: u8, last: u8, len: usize) -> usize {
    let val = (first as usize) * 1176
        + (mid as usize) * 2141
        + (last as usize) * 150
        + len * 3013;
    (val ^ (val >> 1)) & KW_SLOT_MASK
}

#[inline(always)]
fn skip_non_tokens_avx2(ptr: &[u8], mut i: usize) -> usize {
    debug_assert!(i <= ptr.len());
    #[cfg(target_feature = "avx2")]
    {
        use safe_arch::*;
        let lut_lo = m256i::from([
            3_i8, 3, 11, 11, 3, 3, 3, 11, 3, 3, 19, 1, 1, 1, 1, 13, 3, 3, 11, 11, 3, 3, 3, 11, 3,
            3, 19, 1, 1, 1, 1, 13,
        ]);
        let lut_hi = m256i::from([
            16_i8, 0, 8, 0, 1, 6, 1, 2, 0, 0, 0, 0, 0, 0, 0, 0, 16, 0, 8, 0, 1, 6, 1, 2, 0, 0, 0,
            0, 0, 0, 0, 0,
        ]);
        let mask_0f = set_splat_i8_m256i(0x0F);
        let zero = zeroed_m256i();

        let (chunks, _) = ptr[i..].as_chunks::<32>();
        for &chunk in chunks {
            let v = m256i::from(chunk);
            let hi = shr_imm_u16_m256i::<4>(v) & mask_0f;
            let m_lo = shuffle_av_i8z_half_m256i(lut_lo, v);
            let m_hi = shuffle_av_i8z_half_m256i(lut_hi, hi);
            let hit = m_lo & m_hi;
            let is_zero = cmp_eq_mask_i8_m256i(hit, zero);
            let mask = !(move_mask_i8_m256i(is_zero) as u32);
            if mask != 0 {
                return i + mask.trailing_zeros() as usize;
            }
            i += 32;
        }
    }
    while i < ptr.len() && TOKEN_KIND[ptr[i] as usize] <= TOK_CODE && ptr[i] != b'\n' {
        i += 1;
    }
    i
}

#[cfg(test)]
#[inline(always)]
pub fn is_odin_extension(name: &[u8]) -> bool {
    let len = name.len();
    if len < 6 || name[len - 5] != b'.' {
        return false;
    }
    let o = name[len - 4] | 0x20;
    let d = name[len - 3] | 0x20;
    let i = name[len - 2] | 0x20;
    let n = name[len - 1] | 0x20;
    o == b'o' && d == b'd' && i == b'i' && n == b'n'
}

pub fn parse_into(content: &[u8], acc: &mut ParseResult<KEYWORD_COUNT>) {
    let content_len = content.len();
    if content_len == 0 {
        return;
    }

    const STATE_NONE: u8 = 0;
    const STATE_CODE: u8 = 1;
    const STATE_COMMENT: u8 = 2;

    let mut line_state = STATE_NONE;
    let mut i = 0usize;
    while i < content_len {
        if line_state == STATE_NONE {
            while i < content_len
                && (content[i] == b' ' || content[i] == b'\t' || content[i] == b'\r')
            {
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
            if i < content_len
                && TOKEN_KIND[content[i] as usize] <= TOK_CODE
                && content[i] != b'\n'
            {
                i = skip_non_tokens_avx2(content, i);
            }
            if i >= content_len {
                break;
            }
        }

        let b = content[i];
        let kind = TOKEN_KIND[b as usize];
        match kind {
            TOK_NL => {
                if (line_state & STATE_CODE) != 0 {
                    acc.code_lines += 1;
                } else if (line_state & STATE_COMMENT) != 0 {
                    acc.comment_lines += 1;
                } else {
                    acc.blank_lines += 1;
                }
                acc.lines_processed += 1;
                line_state = STATE_NONE;
                i += 1;
            }
            TOK_IDENT => {
                line_state |= STATE_CODE;
                let start = i;
                let end = skip_ident_cont_avx2(content, i + 1);
                i = end;
                let len = end - start;
                if len.wrapping_sub(2) <= KW_MAX_LEN - 2 {
                    let first = content[start];
                    let mid = content[start + len / 2];
                    let last = content[end - 1];
                    let h = kw_hash(first, mid, last, len);
                    let slot = KW_SLOT[h];
                    if slot != KW_EMPTY {
                        let idx = slot as usize;
                        if len == KEYWORD_LENS[idx] as usize
                            && kw_match_2_to_16(&content[start..end], KEYWORD_NAMES[idx].as_bytes())
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
                            if (line_state & STATE_CODE) != 0 {
                                acc.code_lines += 1;
                            } else if (line_state & STATE_COMMENT) != 0 {
                                acc.comment_lines += 1;
                            } else {
                                acc.blank_lines += 1;
                            }
                            acc.lines_processed += 1;
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
                        line_state = STATE_NONE;
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
                if i + 2 < content_len && content[i + 1] == b'"' && content[i + 2] == b'"' {
                    let end_i = skip_triple_literal(content, i + 3, b'"');
                    let nl = count_newlines_avx2(&content[i + 3..end_i]);
                    if nl > 0 {
                        acc.code_lines += nl as i64;
                        acc.lines_processed += nl as i64;
                    }
                    i = end_i;
                    continue;
                }
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
                let end_i = skip_literal(content, i + 1, b'\'');
                let nl = count_newlines_avx2(&content[i + 1..end_i]);
                if nl > 0 {
                    acc.code_lines += nl as i64;
                    acc.lines_processed += nl as i64;
                }
                i = end_i;
            }
            TOK_BACKTICK => {
                line_state |= STATE_CODE;
                let end_i = match find_byte_avx2(&content[i + 1..], b'`') {
                    Some(pos) => i + 1 + pos + 1,
                    None => content_len,
                };
                let nl = count_newlines_avx2(&content[i + 1..end_i]);
                if nl > 0 {
                    acc.code_lines += nl as i64;
                    acc.lines_processed += nl as i64;
                }
                i = end_i;
            }
            _ => {
                line_state |= STATE_CODE;
                i += 1;
            }
        }
    }
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
            assert_eq!(KW_SLOT[h], i as u8);

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
    }
}
