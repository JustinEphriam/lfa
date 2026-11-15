use super::{
    compute_keyword_lens, compute_kw_max_len, compute_kw_min_len, count_newlines_avx2,
    handle_nested_c_slash_comments, handle_quoted_literal, make_token_kind, make_token_lut_lo,
    match_keyword_with, rust, skip_ident_cont_avx2, unified_parse_into, IDENT_CONT, LangLexer,
    ParseResult, STATE_CODE,
};

pub const KEYWORD_COUNT: usize = 5;
const KW_SLOT_MASK: usize = 31;
const KW_EMPTY: u8 = 255;

pub const KEYWORD_NAMES: [&str; KEYWORD_COUNT] = [
    "false",
    "inf",
    "nan",
    "null",
    "true",
];

pub const KEYWORD_LENS: [u8; KEYWORD_COUNT] = compute_keyword_lens(&KEYWORD_NAMES);
pub const KW_MIN_LEN: usize = compute_kw_min_len(&KEYWORD_NAMES);
pub const KW_MAX_LEN: usize = compute_kw_max_len(&KEYWORD_NAMES);

const _: () = assert!(KEYWORD_NAMES.len() == KEYWORD_COUNT);
const _: () = assert!(KEYWORD_LENS.len() == KEYWORD_COUNT);
const _: () = assert!(KW_MIN_LEN >= 3 && KW_MAX_LEN <= 5);

#[inline(always)]
const fn kw_hash(cand: &[u8], len: usize) -> usize {
    let b0 = cand[0] as usize;
    let bmid = cand[len / 2] as usize;
    let blast = cand[len - 1] as usize;
    let val = len.wrapping_mul(9594)
        .wrapping_add(b0.wrapping_mul(1078))
        .wrapping_add(bmid.wrapping_mul(9074))
        .wrapping_add(blast.wrapping_mul(661));
    (val ^ (val >> 2)) & KW_SLOT_MASK
}

const KW_TABLE: [u8; 32] = {
    let mut table = [KW_EMPTY; 32];
    let mut i = 0;
    while i < KEYWORD_COUNT {
        let name = KEYWORD_NAMES[i].as_bytes();
        let len = KEYWORD_LENS[i] as usize;
        assert!(name.len() == len);
        assert!(len >= KW_MIN_LEN && len <= KW_MAX_LEN);
        let h = kw_hash(name, len);
        assert!(table[h] == KW_EMPTY);
        table[h] = i as u8;
        i += 1;
    }
    table
};

const TOK_IDENT: u8 = 3;
const TOK_SLASH: u8 = 4;
const TOK_QUOTE: u8 = 5;

pub const TOKEN_KIND: [u8; 256] = make_token_kind(
    TOK_IDENT,
    &[
        (b'/', TOK_SLASH),
        (b'"', TOK_QUOTE),
    ],
);

pub const TOKEN_LUT_LO: [i8; 32] = make_token_lut_lo(&TOKEN_KIND);

#[cfg(test)]
#[inline(always)]
pub fn is_kdl_extension(name: &[u8]) -> bool {
    super::detect_language(name, super::LANG_ALL_MASK) == Some(super::LangId::Kdl)
}

pub struct KdlLexer;

impl LangLexer<KEYWORD_COUNT> for KdlLexer {
    const TOKEN_KIND: [u8; 256] = TOKEN_KIND;
    const TOKEN_LUT_LO: [i8; 32] = TOKEN_LUT_LO;
    const HAS_DOLLAR_IDENT: bool = false;
    const SUPPORTS_TEMPLATES: bool = false;
    const HAS_SPECIAL_IDENT: bool = true;

    #[inline(always)]
    fn match_keyword(cand: &[u8]) -> Option<usize> {
        match_keyword_with(
            cand,
            KW_MIN_LEN,
            KW_MAX_LEN,
            &KW_TABLE,
            &KEYWORD_LENS,
            &KEYWORD_NAMES,
            |c| kw_hash(c, c.len()),
        )
    }

    #[inline(always)]
    fn handle_ident(
        content: &[u8],
        start: usize,
        end: usize,
        _line_state: &mut u8,
        acc: &mut ParseResult<KEYWORD_COUNT>,
    ) -> usize {
        let content_len = content.len();
        let cand = &content[start..end];
        if end < content_len {
            let next_b = content[end];
            if next_b == b'#' && cand == b"r" {
                let mut hash_end = end + 1;
                while hash_end < content_len && content[hash_end] == b'#' {
                    hash_end += 1;
                }
                let hashes = hash_end - end;
                if hashes == 1
                    && hash_end < content_len
                    && IDENT_CONT[content[hash_end] as usize] != 0
                {
                    return skip_ident_cont_avx2(content, hash_end + 1);
                }
                if hash_end < content_len && content[hash_end] == b'"' {
                    let next_i = rust::skip_rust_raw_string(content, hash_end + 1, hashes);
                    let nl = count_newlines_avx2(&content[hash_end + 1..next_i]);
                    if nl > 0 {
                        acc.code_lines += nl as i64;
                        acc.lines_processed += nl as i64;
                    }
                    return next_i;
                }
            } else if next_b == b'"' && cand == b"r" {
                let next_i = rust::skip_rust_raw_string(content, end + 1, 0);
                let nl = count_newlines_avx2(&content[end + 1..next_i]);
                if nl > 0 {
                    acc.code_lines += nl as i64;
                    acc.lines_processed += nl as i64;
                }
                return next_i;
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
        match kind {
            TOK_SLASH => handle_nested_c_slash_comments(content, i, line_state, acc),
            TOK_QUOTE => handle_quoted_literal(content, i, b'"', line_state, acc),
            _ => {
                *line_state |= STATE_CODE;
                i + 1
            }
        }
    }
}

#[inline(always)]
pub fn parse_into(content: &[u8], acc: &mut ParseResult<KEYWORD_COUNT>) {
    unified_parse_into::<KEYWORD_COUNT, KdlLexer>(content, acc);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyword_table_and_mph() {
        assert_eq!(KEYWORD_COUNT, 5);
        for i in 0..KEYWORD_COUNT {
            let kw = KEYWORD_NAMES[i];
            let len = kw.len();
            assert_eq!(len, KEYWORD_LENS[i] as usize);
            let bytes = kw.as_bytes();
            let h = kw_hash(bytes, len);
            assert_eq!(KW_TABLE[h], i as u8);

            let mut res = ParseResult::default();
            parse_into(bytes, &mut res);
            assert_eq!(res.total_keywords, 1);
            assert_eq!(res.keyword_counts[i], 1);
            assert_eq!(res.lines_processed, 1);
        }

        let non_keywords = ["falses", "nulls", "trues", "nanoseconds"];
        for s in non_keywords {
            let mut res = ParseResult::default();
            parse_into(s.as_bytes(), &mut res);
            assert_eq!(res.total_keywords, 0);
        }
    }

    #[test]
    fn test_comments_and_strings() {
        let text = concat!(
            "// line comment\n",
            "/* nested /* block */ comment */\n",
            "node key=\"value\" active=true {\n",
            "    child false null\n",
            "    raw r#\"raw \"# string\"#\n",
            "}\n"
        );
        let mut res = ParseResult::default();
        parse_into(text.as_bytes(), &mut res);
        assert_eq!(res.lines_processed, 6);
        assert_eq!(res.comment_lines, 2);
        assert_eq!(res.code_lines, 4);
        assert_eq!(res.blank_lines, 0);
    }

    #[test]
    fn test_lines_code_comments_blanks() {
        let text = concat!(
            "\n",
            "// header\n",
            "\n",
            "package {\n",
            "    name \"my-pkg\"\n",
            "}\n",
            "\n"
        );
        let mut res = ParseResult::default();
        parse_into(text.as_bytes(), &mut res);
        assert_eq!(res.lines_processed, 7);
        assert_eq!(res.blank_lines, 3);
        assert_eq!(res.comment_lines, 1);
        assert_eq!(res.code_lines, 3);
    }

    #[test]
    fn test_extensions() {
        assert!(is_kdl_extension(b"config.kdl"));
        assert!(is_kdl_extension(b"layout.kdl"));
        assert!(!is_kdl_extension(b"main.rs"));
    }
}
