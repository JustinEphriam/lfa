use super::{
    compute_keyword_lens, compute_kw_max_len, compute_kw_min_len, count_newlines_avx2,
    find_byte_avx2, finish_block_comment, finish_line_comment, handle_quoted_literal,
    kw_hash_fml_add, make_kw_table_fml_add, make_token_kind, make_token_lut_lo, match_keyword_with,
    unified_parse_into, LangLexer, ParseResult, STATE_CODE,
};

#[cfg(test)]
use super::{skip_ident_cont_avx2, skip_non_tokens_with_lut_avx2, TOK_CODE};

pub const KEYWORD_COUNT: usize = 22;
const KW_SLOT_MASK: usize = 255;

pub const KEYWORD_NAMES: [&str; KEYWORD_COUNT] = [
    "and",
    "break",
    "do",
    "else",
    "elseif",
    "end",
    "false",
    "for",
    "function",
    "goto",
    "if",
    "in",
    "local",
    "nil",
    "not",
    "or",
    "repeat",
    "return",
    "then",
    "true",
    "until",
    "while",
];

pub const KEYWORD_LENS: [u8; KEYWORD_COUNT] = compute_keyword_lens(&KEYWORD_NAMES);
pub const KW_MIN_LEN: usize = compute_kw_min_len(&KEYWORD_NAMES);
pub const KW_MAX_LEN: usize = compute_kw_max_len(&KEYWORD_NAMES);

const TOK_IDENT: u8 = 3;
const TOK_MINUS: u8 = 4;
const TOK_QUOTE: u8 = 5;
const TOK_APOSTROPHE: u8 = 6;
const TOK_LBRACKET: u8 = 7;

#[inline(always)]
const fn kw_hash(first: u8, mid: u8, last: u8, len: usize) -> usize {
    kw_hash_fml_add(first, mid, last, len, [1, 1, 1, 47], 5, KW_SLOT_MASK)
}

const KW_TABLE: [u8; 256] = make_kw_table_fml_add(&KEYWORD_NAMES, [1, 1, 1, 47], 5);

pub const TOKEN_KIND: [u8; 256] = make_token_kind(
    TOK_IDENT,
    &[
        (b'-', TOK_MINUS),
        (b'"', TOK_QUOTE),
        (b'\'', TOK_APOSTROPHE),
        (b'[', TOK_LBRACKET),
    ],
);

pub const TOKEN_LUT_LO: [i8; 32] = make_token_lut_lo(&TOKEN_KIND);

#[inline(always)]
fn skip_lua_long_bracket(content: &[u8], start_bracket: usize) -> Option<usize> {
    let content_len = content.len();
    let mut eq_count = 0;
    let mut p = start_bracket + 1;
    while p < content_len && content[p] == b'=' {
        eq_count += 1;
        p += 1;
    }
    if p >= content_len || content[p] != b'[' {
        return None;
    }
    p += 1;
    while p < content_len {
        let Some(pos) = find_byte_avx2(&content[p..], b']') else {
            return Some(content_len);
        };
        let close_bracket = p + pos;
        let mut close_p = close_bracket + 1;
        let mut close_eq = 0;
        while close_p < content_len && content[close_p] == b'=' {
            close_eq += 1;
            close_p += 1;
        }
        if close_eq == eq_count && close_p < content_len && content[close_p] == b']' {
            return Some(close_p + 1);
        }
        p = close_bracket + 1;
    }
    Some(content_len)
}

#[cfg(test)]
#[inline(always)]
pub fn is_lua_extension(name: &[u8]) -> bool {
    super::detect_language(name, super::LANG_ALL_MASK) == Some(super::LangId::Lua)
}

pub struct LuaLexer;

impl LangLexer<KEYWORD_COUNT> for LuaLexer {
    const TOKEN_KIND: [u8; 256] = TOKEN_KIND;
    const TOKEN_LUT_LO: [i8; 32] = TOKEN_LUT_LO;
    const HAS_DOLLAR_IDENT: bool = false;
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
            TOK_MINUS => {
                if i + 1 < content_len && content[i + 1] == b'-' {
                    if i + 2 < content_len
                        && content[i + 2] == b'['
                        && let Some(end_i) = skip_lua_long_bracket(content, i + 2)
                    {
                        return finish_block_comment(content, i + 2, end_i, line_state, acc);
                    }
                    return finish_line_comment(content, i + 2, line_state, acc);
                }
                *line_state |= STATE_CODE;
                i + 1
            }
            TOK_LBRACKET => {
                *line_state |= STATE_CODE;
                if let Some(end_i) = skip_lua_long_bracket(content, i) {
                    let nl = count_newlines_avx2(&content[i..end_i]);
                    if nl > 0 {
                        acc.code_lines += nl as i64;
                        acc.lines_processed += nl as i64;
                    }
                    return end_i;
                }
                i + 1
            }
            TOK_QUOTE | TOK_APOSTROPHE => {
                handle_quoted_literal(content, i, content[i], line_state, acc)
            }
            _ => {
                *line_state |= STATE_CODE;
                i + 1
            }
        }
    }
}

pub fn parse_into(content: &[u8], acc: &mut ParseResult<KEYWORD_COUNT>) {
    unified_parse_into::<KEYWORD_COUNT, LuaLexer>(content, acc);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn find_kw_id(name: &str) -> usize {
        KEYWORD_NAMES.iter().position(|&kw| kw == name).unwrap()
    }

    #[test]
    fn test_lua_extensions() {
        assert!(is_lua_extension(b"main.lua"));
        assert!(is_lua_extension(b"path/to/script.LUA"));
        assert!(!is_lua_extension(b"main.py"));
        assert!(!is_lua_extension(b".lua"));
    }

    #[test]
    fn test_lua_keyword_table_and_mph() {
        assert_eq!(KEYWORD_COUNT, 22);
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
            assert!(!seen[h], "collision at slot {} for kw {}", h, kw);
            seen[h] = true;
            assert_eq!(KW_TABLE[h], i as u8);
        }

        let non_keywords = ["functionality", "fore", "els", "whiles", "locale", "nilify"];
        for s in non_keywords {
            let mut res = ParseResult::default();
            parse_into(s.as_bytes(), &mut res);
            assert_eq!(res.total_keywords, 0, "False positive on: {}", s);
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
            if super::super::IDENT_CONT[b as usize] == 0 {
                assert_eq!(id_pos, 19);
            } else {
                assert_eq!(id_pos, block.len());
            }
        }
    }

    #[test]
    fn test_lua_comments_and_strings() {
        let src = concat!(
            "local x = 10 -- short comment\n",
            "--[[ multi\n",
            "     line\n",
            "     comment ]] local y = 20\n",
            "--[==[ level 2\n",
            "       comment ]==]\n",
            "local s = [[ long\n",
            "string ]]\n",
            "if x > 5 then return end\n"
        );
        let mut res = ParseResult::default();
        parse_into(src.as_bytes(), &mut res);
        assert_eq!(res.keyword_counts[find_kw_id("local")], 3);
        assert_eq!(res.keyword_counts[find_kw_id("if")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("then")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("return")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("end")], 1);
        assert_eq!(res.lines_processed, 9);
        assert_eq!(res.lines_processed, res.code_lines + res.comment_lines + res.blank_lines);
    }

    #[test]
    fn test_lua_lines_code_comments_blanks() {
        let src = "local a = 1\n\n-- comment\n\nreturn a\n";
        let mut res = ParseResult::default();
        parse_into(src.as_bytes(), &mut res);
        assert_eq!(res.lines_processed, 5);
        assert_eq!(res.code_lines, 2);
        assert_eq!(res.comment_lines, 1);
        assert_eq!(res.blank_lines, 2);
    }
}
