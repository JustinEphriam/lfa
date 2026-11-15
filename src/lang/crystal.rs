use super::{
    compute_keyword_lens, compute_kw_max_len, compute_kw_min_len, finish_line_comment,
    handle_quoted_literal, make_token_kind, make_token_lut_lo, match_keyword_with,
    unified_parse_into, LangLexer, ParseResult, STATE_CODE,
};

pub const KEYWORD_COUNT: usize = 57;
const KW_SLOT_MASK: usize = 127;
const KW_EMPTY: u8 = 255;

pub const KEYWORD_NAMES: [&str; KEYWORD_COUNT] = [
    "__DIR__",
    "__FILE__",
    "__LINE__",
    "abstract",
    "alias",
    "annotation",
    "as",
    "asm",
    "begin",
    "break",
    "case",
    "class",
    "def",
    "do",
    "else",
    "elsif",
    "end",
    "ensure",
    "enum",
    "extend",
    "false",
    "for",
    "fun",
    "if",
    "in",
    "include",
    "instance_sizeof",
    "lib",
    "macro",
    "module",
    "next",
    "nil",
    "of",
    "out",
    "pointerof",
    "private",
    "protected",
    "rescue",
    "return",
    "select",
    "self",
    "sizeof",
    "struct",
    "super",
    "then",
    "true",
    "type",
    "typeof",
    "uninitialized",
    "union",
    "unless",
    "until",
    "verbatim",
    "when",
    "while",
    "with",
    "yield",
];

pub const KEYWORD_LENS: [u8; KEYWORD_COUNT] = compute_keyword_lens(&KEYWORD_NAMES);
pub const KW_MIN_LEN: usize = compute_kw_min_len(&KEYWORD_NAMES);
pub const KW_MAX_LEN: usize = compute_kw_max_len(&KEYWORD_NAMES);

const _: () = assert!(KEYWORD_NAMES.len() == KEYWORD_COUNT);
const _: () = assert!(KEYWORD_LENS.len() == KEYWORD_COUNT);
const _: () = assert!(KW_MIN_LEN >= 2 && KW_MAX_LEN <= 15);

#[inline(always)]
const fn kw_hash(cand: &[u8], len: usize) -> usize {
    let b0 = cand[0] as usize;
    let bmid = cand[len / 2] as usize;
    let blast = cand[len - 1] as usize;
    let val = len.wrapping_mul(6525)
        .wrapping_add(b0.wrapping_mul(8563))
        .wrapping_add(bmid.wrapping_mul(9650))
        .wrapping_add(blast.wrapping_mul(2829));
    (val ^ (val >> 2)) & KW_SLOT_MASK
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
const TOK_HASH: u8 = 4;
const TOK_QUOTE: u8 = 5;
const TOK_APOSTROPHE: u8 = 6;

pub const TOKEN_KIND: [u8; 256] = make_token_kind(
    TOK_IDENT,
    &[
        (b'#', TOK_HASH),
        (b'"', TOK_QUOTE),
        (b'\'', TOK_APOSTROPHE),
    ],
);

pub const TOKEN_LUT_LO: [i8; 32] = make_token_lut_lo(&TOKEN_KIND);

#[cfg(test)]
#[inline(always)]
pub fn is_crystal_extension(name: &[u8]) -> bool {
    super::detect_language(name, super::LANG_ALL_MASK) == Some(super::LangId::Crystal)
}

pub struct CrystalLexer;

impl LangLexer<KEYWORD_COUNT> for CrystalLexer {
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
            |c| kw_hash(c, c.len()),
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
        match kind {
            TOK_HASH => finish_line_comment(content, i, line_state, acc),
            TOK_QUOTE => handle_quoted_literal(content, i, b'"', line_state, acc),
            TOK_APOSTROPHE => handle_quoted_literal(content, i, b'\'', line_state, acc),
            _ => {
                *line_state |= STATE_CODE;
                i + 1
            }
        }
    }
}

#[inline(always)]
pub fn parse_into(content: &[u8], acc: &mut ParseResult<KEYWORD_COUNT>) {
    unified_parse_into::<KEYWORD_COUNT, CrystalLexer>(content, acc);
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn find_kw_id(name: &str) -> usize {
        KEYWORD_NAMES.iter().position(|&kw| kw == name).unwrap()
    }

    #[test]
    fn test_keyword_table_and_mph() {
        assert_eq!(KEYWORD_COUNT, 57);
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

        let non_keywords = ["abstracts", "defx", "end_", "classes", "modules", "foo"];
        for s in non_keywords {
            let mut res = ParseResult::default();
            parse_into(s.as_bytes(), &mut res);
            assert_eq!(res.total_keywords, 0);
        }
    }

    #[test]
    fn test_extensions() {
        assert!(is_crystal_extension(b"main.cr"));
        assert!(is_crystal_extension(b"UPPER.CR"));
        assert!(is_crystal_extension(b"a/b/c/foo.bar.cr"));
        assert!(!is_crystal_extension(b"main.c"));
        assert!(!is_crystal_extension(b"main.rs"));
    }

    #[test]
    fn test_comments_and_strings() {
        let src = concat!(
            "# Crystal line comment with if def class\n",
            "class Foo\n",
            "  def bar\n",
            "    s = \"string with def end\"\n",
            "    c = 'a'\n",
            "    return 42\n",
            "  end\n",
            "end\n"
        );
        let mut res = ParseResult::default();
        parse_into(src.as_bytes(), &mut res);
        assert_eq!(res.lines_processed, 8);
        assert_eq!(res.comment_lines, 1);
        assert_eq!(res.code_lines, 7);
        assert_eq!(res.keyword_counts[find_kw_id("class")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("def")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("end")], 2);
        assert_eq!(res.keyword_counts[find_kw_id("return")], 1);
    }

    #[test]
    fn test_lines_code_comments_blanks() {
        let src = concat!(
            "# comment line\n",
            "\n",
            "   \t  \r\n",
            "def foo # inline comment\n",
            "  42\n",
            "end\n"
        );
        let mut res = ParseResult::default();
        parse_into(src.as_bytes(), &mut res);
        assert_eq!(res.lines_processed, 6);
        assert_eq!(res.blank_lines, 2);
        assert_eq!(res.comment_lines, 1);
        assert_eq!(res.code_lines, 3);
    }
}
