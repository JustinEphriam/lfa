use super::{count_newlines_avx2, find_byte_avx2, find_star_slash, skip_literal, ParseResult};

pub const KEYWORD_COUNT: usize = 25;
const KW_SLOT_MASK: usize = 255;
const KW_MAX_LEN: usize = 11;
const KW_EMPTY: u8 = 255;

pub const KEYWORD_NAMES: [&str; KEYWORD_COUNT] = [
    "break",
    "case",
    "chan",
    "const",
    "continue",
    "default",
    "defer",
    "else",
    "fallthrough",
    "for",
    "func",
    "go",
    "goto",
    "if",
    "import",
    "interface",
    "map",
    "package",
    "range",
    "return",
    "select",
    "struct",
    "switch",
    "type",
    "var",
];

pub const KEYWORD_LENS: [u8; KEYWORD_COUNT] = [
    5, 4, 4, 5, 8, 7, 5, 4, 11, 3, 4, 2, 4, 2, 6, 9, 3, 7, 5, 6, 6, 6, 6, 4, 3,
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

pub const IDENT_CONT: [u8; 256] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0,
    0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 0, 0, 0, 0, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

pub const KW_SLOT: [u8; 256] = [
    255, 255, 255, 255, 255, 255, 20, 255, 255, 255, 255, 6, 255, 255, 255, 255,
    255, 255, 255, 255, 16, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    14, 255, 255, 255, 255, 255, 255, 255, 255, 12, 255, 255, 255, 5, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 24, 255, 255, 10, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 17, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 4, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 21, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 1, 255, 255, 255, 255, 255, 255,
    255, 255, 18, 2, 255, 255, 255, 255, 3, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 9, 255, 255, 255, 255, 15, 255, 7, 255, 255, 0, 255, 255,
    255, 255, 255, 13, 255, 255, 255, 255, 23, 255, 255, 255, 255, 255, 255, 8,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 22, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 11, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 19, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
];

#[inline(always)]
pub fn kw_hash(first: u8, second: u8, last: u8, len: usize) -> usize {
    let val = (first as usize) * 2066
        + (second as usize) * 654
        + (last as usize) * 3810
        + len * 320;
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

#[inline(always)]
fn skip_ident_cont_avx2(ptr: &[u8], mut i: usize) -> usize {
    debug_assert!(i <= ptr.len());
    #[cfg(target_feature = "avx2")]
    {
        use safe_arch::*;
        let lut_lo = m256i::from([
            10_i8, 11, 11, 11, 11, 11, 11, 11, 11, 11, 3, 1, 1, 1, 1, 5, 10, 11, 11, 11, 11, 11,
            11, 11, 11, 11, 3, 1, 1, 1, 1, 5,
        ]);
        let lut_hi = m256i::from([
            0_i8, 0, 0, 8, 1, 6, 1, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 1, 6, 1, 2, 0, 0, 0, 0,
            0, 0, 0, 0,
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
            let is_non_ident = cmp_eq_mask_i8_m256i(hit, zero);
            let mask = move_mask_i8_m256i(is_non_ident) as u32;
            if mask != 0 {
                return i + mask.trailing_zeros() as usize;
            }
            i += 32;
        }
    }
    while i < ptr.len() && IDENT_CONT[ptr[i] as usize] != 0 {
        i += 1;
    }
    i
}

#[inline(always)]
pub fn is_go_extension(name: &[u8]) -> bool {
    let len = name.len();
    if len < 4 || name[len - 3] != b'.' {
        return false;
    }
    let g = name[len - 2] | 0x20;
    let o = name[len - 1] | 0x20;
    g == b'g' && o == b'o'
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
                match line_state {
                    STATE_NONE => acc.blank_lines += 1,
                    STATE_COMMENT => acc.comment_lines += 1,
                    _ => acc.code_lines += 1,
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
                if (2..=KW_MAX_LEN).contains(&len) {
                    let first = content[start];
                    let second = content[start + 1];
                    let last = content[end - 1];
                    let h = kw_hash(first, second, last, len);
                    let slot = KW_SLOT[h];
                    if slot != KW_EMPTY {
                        let idx = slot as usize;
                        if len == KEYWORD_LENS[idx] as usize
                            && &content[start..end] == KEYWORD_NAMES[idx].as_bytes()
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
                        if line_state == STATE_NONE {
                            line_state = STATE_COMMENT;
                        }
                        let block_start = i + 2;
                        if let Some(pos) = find_star_slash(&content[block_start..]) {
                            let end_i = block_start + pos + 2;
                            let nls = count_newlines_avx2(&content[block_start..end_i - 2]);
                            if nls > 0 {
                                acc.comment_lines += nls as i64;
                                acc.lines_processed += nls as i64;
                            }
                            i = end_i;
                        } else {
                            let rem_nls = count_newlines_avx2(&content[block_start..]);
                            if rem_nls > 0 {
                                acc.comment_lines += rem_nls as i64;
                                acc.lines_processed += rem_nls as i64;
                            }
                            i = content_len;
                        }
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
    fn test_go_keyword_table_and_mph() {
        assert_eq!(KEYWORD_COUNT, 25);
        let mut seen = [false; 256];
        for i in 0..KEYWORD_COUNT {
            let kw = KEYWORD_NAMES[i];
            let len = kw.len();
            assert_eq!(len, KEYWORD_LENS[i] as usize);
            assert!((2..=KW_MAX_LEN).contains(&len));

            let bytes = kw.as_bytes();
            let first = bytes[0];
            let second = bytes[1];
            let last = bytes[len - 1];
            let h = kw_hash(first, second, last, len);
            assert!(!seen[h]);
            seen[h] = true;
            assert_eq!(KW_SLOT[h], i as u8);

            let mut res = ParseResult::default();
            parse_into(bytes, &mut res);
            assert_eq!(res.total_keywords, 1);
            assert_eq!(res.keyword_counts[i], 1);
        }

        let non_keywords = [
            "main", "foo", "fmt", "Println", "string", "int", "true", "false", "nil", "make",
            "len", "break1", "gos", "ifs",
        ];
        for &nk in &non_keywords {
            let mut res = ParseResult::default();
            parse_into(nk.as_bytes(), &mut res);
            assert_eq!(res.total_keywords, 0);
        }
    }

    #[test]
    fn test_go_extensions() {
        assert!(is_go_extension(b"main.go"));
        assert!(is_go_extension(b"server.GO"));
        assert!(is_go_extension(b"path/to/file.go"));
        assert!(!is_go_extension(b"main.g"));
        assert!(!is_go_extension(b"main.c"));
        assert!(!is_go_extension(b"go"));
        assert!(!is_go_extension(b".go"));
    }

    #[test]
    fn test_go_comments_and_strings() {
        let src = b"package main\n\n// line comment\nfunc main() {\n    /* block\n       comment */\n    s := \"hello // not a comment\"\n    r := 'a'\n    return\n}\n";
        let mut res = ParseResult::default();
        parse_into(src, &mut res);

        assert_eq!(res.keyword_counts[find_kw_id("package")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("func")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("return")], 1);
        assert_eq!(res.total_keywords, 3);
        assert_eq!(res.lines_processed, 10);
        assert_eq!(res.code_lines, 6);
        assert_eq!(res.comment_lines, 3);
        assert_eq!(res.blank_lines, 1);
    }

    #[test]
    fn test_go_raw_strings() {
        let src = b"package main\n\nconst raw = `line 1\nline 2 with func and var\nline 3`\n\nvar x = 1\n";
        let mut res = ParseResult::default();
        parse_into(src, &mut res);

        assert_eq!(res.keyword_counts[find_kw_id("package")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("const")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("var")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("func")], 0);
        assert_eq!(res.total_keywords, 3);
        assert_eq!(res.code_lines, 5);
        assert_eq!(res.comment_lines, 0);
        assert_eq!(res.blank_lines, 2);
    }

    #[test]
    fn test_go_lines_code_comments_blanks() {
        let src = b"// Copyright\n\npackage main\n\nfunc main() {\n\n}\n";
        let mut res = ParseResult::default();
        parse_into(src, &mut res);

        assert_eq!(res.lines_processed, 7);
        assert_eq!(res.code_lines, 3);
        assert_eq!(res.comment_lines, 1);
        assert_eq!(res.blank_lines, 3);
    }

    #[test]
    fn test_go_avx2_boundaries_and_eof() {
        for len in [31, 32, 33, 63, 64, 65, 127, 128, 129] {
            let mut buf = vec![b' '; len];
            buf.extend_from_slice(b"func foo() {\n    return\n}\n");
            let mut res = ParseResult::default();
            parse_into(&buf, &mut res);
            assert_eq!(res.keyword_counts[find_kw_id("func")], 1);
            assert_eq!(res.keyword_counts[find_kw_id("return")], 1);
            assert_eq!(res.total_keywords, 2);
        }

        let mut res = ParseResult::default();
        parse_into(b"/* unclosed comment", &mut res);
        assert_eq!(res.total_keywords, 0);

        let mut res2 = ParseResult::default();
        parse_into(b"`unclosed raw string", &mut res2);
        assert_eq!(res2.total_keywords, 0);
    }
}
