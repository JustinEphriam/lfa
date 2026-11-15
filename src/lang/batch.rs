use super::{
    compute_keyword_lens, compute_kw_max_len, compute_kw_min_len, finish_line_comment,
    handle_quoted_literal, make_token_kind, make_token_lut_lo, match_keyword_caseless_with,
    unified_parse_into, LangLexer, ParseResult, STATE_CODE,
};

pub const KEYWORD_COUNT: usize = 49;
const KW_SLOT_MASK: usize = 127;
const KW_EMPTY: u8 = 255;

pub const KEYWORD_NAMES: [&str; KEYWORD_COUNT] = [
    "assoc",
    "attrib",
    "call",
    "cd",
    "chcp",
    "chdir",
    "cls",
    "cmd",
    "color",
    "copy",
    "date",
    "del",
    "dir",
    "echo",
    "endlocal",
    "erase",
    "errorlevel",
    "exist",
    "exit",
    "find",
    "findstr",
    "for",
    "ftype",
    "goto",
    "if",
    "in",
    "md",
    "mkdir",
    "move",
    "not",
    "off",
    "on",
    "pause",
    "popd",
    "pushd",
    "rd",
    "ren",
    "rename",
    "rmdir",
    "set",
    "setlocal",
    "shift",
    "start",
    "time",
    "title",
    "type",
    "ver",
    "verify",
    "vol",
];

pub const KEYWORD_LENS: [u8; KEYWORD_COUNT] = compute_keyword_lens(&KEYWORD_NAMES);
pub const KW_MIN_LEN: usize = compute_kw_min_len(&KEYWORD_NAMES);
pub const KW_MAX_LEN: usize = compute_kw_max_len(&KEYWORD_NAMES);

const _: () = assert!(KEYWORD_NAMES.len() == KEYWORD_COUNT);
const _: () = assert!(KEYWORD_LENS.len() == KEYWORD_COUNT);
const _: () = assert!(KW_MIN_LEN >= 2 && KW_MAX_LEN <= 10);

#[inline(always)]
const fn kw_hash(cand: &[u8], len: usize) -> usize {
    let b0 = cand[0] as usize;
    let bmid = cand[len / 2] as usize;
    let blast = cand[len - 1] as usize;
    let val = len.wrapping_mul(2379)
        .wrapping_add(b0.wrapping_mul(4052))
        .wrapping_add(bmid.wrapping_mul(1637))
        .wrapping_add(blast.wrapping_mul(4232));
    (val ^ (val >> 3)) & KW_SLOT_MASK
}

const KW_TABLE: [u8; 128] = {
    let mut table = [KW_EMPTY; 128];
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
const TOK_COLON: u8 = 4;
const TOK_QUOTE: u8 = 5;

pub const TOKEN_KIND: [u8; 256] = make_token_kind(
    TOK_IDENT,
    &[
        (b':', TOK_COLON),
        (b'"', TOK_QUOTE),
        (b'@', TOK_IDENT),
        (b'%', TOK_IDENT),
    ],
);

pub const TOKEN_LUT_LO: [i8; 32] = make_token_lut_lo(&TOKEN_KIND);

#[cfg(test)]
#[inline(always)]
pub fn is_batch_extension(name: &[u8]) -> bool {
    super::detect_language(name, super::LANG_ALL_MASK) == Some(super::LangId::Batch)
}

pub struct BatchLexer;

impl LangLexer<KEYWORD_COUNT> for BatchLexer {
    const TOKEN_KIND: [u8; 256] = TOKEN_KIND;
    const TOKEN_LUT_LO: [i8; 32] = TOKEN_LUT_LO;
    const HAS_DOLLAR_IDENT: bool = false;
    const SUPPORTS_TEMPLATES: bool = false;
    const HAS_LINE_START_HOOK: bool = true;

    #[inline(always)]
    fn match_keyword(cand: &[u8]) -> Option<usize> {
        let cand = if !cand.is_empty() && (cand[0] == b'@' || cand[0] == b'%') {
            &cand[1..]
        } else {
            cand
        };
        match_keyword_caseless_with(
            cand,
            KW_MIN_LEN,
            KW_MAX_LEN,
            &KW_TABLE,
            &KEYWORD_LENS,
            &KEYWORD_NAMES,
            |c| {
                let b0 = c[0].to_ascii_lowercase() as usize;
                let bmid = c[c.len() / 2].to_ascii_lowercase() as usize;
                let blast = c[c.len() - 1].to_ascii_lowercase() as usize;
                let val = c.len().wrapping_mul(2379)
                    .wrapping_add(b0.wrapping_mul(4052))
                    .wrapping_add(bmid.wrapping_mul(1637))
                    .wrapping_add(blast.wrapping_mul(4232));
                (val ^ (val >> 3)) & KW_SLOT_MASK
            },
        )
    }

    #[inline(always)]
    fn handle_line_start(
        content: &[u8],
        i: usize,
        line_state: &mut u8,
        acc: &mut ParseResult<KEYWORD_COUNT>,
    ) -> usize {
        let content_len = content.len();
        if i + 1 < content_len && content[i] == b':' && content[i + 1] == b':' {
            return finish_line_comment(content, i + 2, line_state, acc);
        }
        if i + 3 <= content_len
            && (content[i] == b'r' || content[i] == b'R')
            && (content[i + 1] == b'e' || content[i + 1] == b'E')
            && (content[i + 2] == b'm' || content[i + 2] == b'M')
            && (i + 3 == content_len
                || content[i + 3] == b' '
                || content[i + 3] == b'\t'
                || content[i + 3] == b'\r'
                || content[i + 3] == b'\n'
                || content[i + 3] == b'/'
                || content[i + 3] == b'\\'
                || content[i + 3] == b'.')
        {
            return finish_line_comment(content, i + 3, line_state, acc);
        }
        i
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
            TOK_COLON => {
                if i + 1 < content.len() && content[i + 1] == b':' {
                    finish_line_comment(content, i + 2, line_state, acc)
                } else {
                    *line_state |= STATE_CODE;
                    i + 1
                }
            }
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
    unified_parse_into::<KEYWORD_COUNT, BatchLexer>(content, acc);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyword_table_and_mph() {
        assert_eq!(KEYWORD_COUNT, 49);
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

            let mut upper = bytes.to_vec();
            upper.make_ascii_uppercase();
            let mut res_upper = ParseResult::default();
            parse_into(&upper, &mut res_upper);
            assert_eq!(res_upper.total_keywords, 1);
            assert_eq!(res_upper.keyword_counts[i], 1);
        }

        let non_keywords = ["assocs", "echos", "titles", "verifyings"];
        for s in non_keywords {
            let mut res = ParseResult::default();
            parse_into(s.as_bytes(), &mut res);
            assert_eq!(res.total_keywords, 0);
        }
    }

    #[test]
    fn test_comments_and_strings() {
        let text = concat!(
            "@echo off\n",
            ":: label comment\n",
            "REM uppercase comment\n",
            "rem lowercase comment\n",
            "set MSG=\"hello :: not comment\"\n",
            "echo %MSG% :: inline comment\n"
        );
        let mut res = ParseResult::default();
        parse_into(text.as_bytes(), &mut res);
        assert_eq!(res.lines_processed, 6);
        assert_eq!(res.comment_lines, 3);
        assert_eq!(res.code_lines, 3);
        assert_eq!(res.blank_lines, 0);
    }

    #[test]
    fn test_lines_code_comments_blanks() {
        let text = concat!(
            "\n",
            ":: header comment\n",
            "\n",
            "@echo off\n",
            "REM another comment\n",
            "goto :eof\n",
            "\n"
        );
        let mut res = ParseResult::default();
        parse_into(text.as_bytes(), &mut res);
        assert_eq!(res.lines_processed, 7);
        assert_eq!(res.blank_lines, 3);
        assert_eq!(res.comment_lines, 2);
        assert_eq!(res.code_lines, 2);
    }

    #[test]
    fn test_extensions() {
        assert!(is_batch_extension(b"build.bat"));
        assert!(is_batch_extension(b"run.cmd"));
        assert!(!is_batch_extension(b"main.rs"));
    }
}
