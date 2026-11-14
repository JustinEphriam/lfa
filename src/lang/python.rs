use super::{
    compute_keyword_lens, compute_kw_max_len, compute_kw_min_len, finish_line_comment,
    handle_triple_or_quoted_literal, kw_hash_fl_add, make_kw_table_fl_add, make_token_kind,
    make_token_lut_lo, match_keyword_with, unified_parse_into, LangLexer, ParseResult, STATE_CODE,
};

#[cfg(test)]
use super::{kw_match_2_to_16, skip_ident_cont_avx2, skip_non_tokens_with_lut_avx2, TOK_CODE};

pub const KEYWORD_COUNT: usize = 37;
const KW_SLOT_MASK: usize = 255;

pub const KEYWORD_NAMES: [&str; KEYWORD_COUNT] = [
    "False",
    "None",
    "True",
    "and",
    "as",
    "assert",
    "async",
    "await",
    "break",
    "case",
    "class",
    "continue",
    "def",
    "del",
    "elif",
    "else",
    "except",
    "finally",
    "for",
    "from",
    "global",
    "if",
    "import",
    "in",
    "is",
    "lambda",
    "match",
    "nonlocal",
    "not",
    "or",
    "pass",
    "raise",
    "return",
    "try",
    "while",
    "with",
    "yield",
];

pub const KEYWORD_LENS: [u8; KEYWORD_COUNT] = compute_keyword_lens(&KEYWORD_NAMES);
pub const KW_MIN_LEN: usize = compute_kw_min_len(&KEYWORD_NAMES);
pub const KW_MAX_LEN: usize = compute_kw_max_len(&KEYWORD_NAMES);

const TOK_IDENT: u8 = 3;
const TOK_HASH: u8 = 4;
const TOK_QUOTE: u8 = 5;
const TOK_APOSTROPHE: u8 = 6;


#[inline(always)]
const fn kw_hash(first: u8, last: u8, len: usize) -> usize {
    kw_hash_fl_add(first, last, len, [1, 8, 12], 0, KW_SLOT_MASK)
}

const KW_TABLE: [u8; 256] = make_kw_table_fl_add(&KEYWORD_NAMES, [1, 8, 12], 0);

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
pub fn is_python_extension(name: &[u8]) -> bool {
    super::detect_language(name, super::LANG_ALL_MASK) == Some(super::LangId::Python)
}

pub struct PythonLexer;

impl LangLexer<KEYWORD_COUNT> for PythonLexer {
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
            |c| kw_hash(c[0], c[c.len() - 1], c.len()),
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
            TOK_HASH => finish_line_comment(content, i + 1, line_state, acc),
            TOK_QUOTE => handle_triple_or_quoted_literal(content, i, b'"', line_state, acc),
            TOK_APOSTROPHE => handle_triple_or_quoted_literal(content, i, b'\'', line_state, acc),
            _ => {
                *line_state |= STATE_CODE;
                i + 1
            }
        }
    }
}

#[inline(always)]
pub fn parse_into(content: &[u8], acc: &mut ParseResult<KEYWORD_COUNT>) {
    unified_parse_into::<KEYWORD_COUNT, PythonLexer>(content, acc);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang::IDENT_CONT;

    pub fn find_kw_id(name: &str) -> usize {
        KEYWORD_NAMES.iter().position(|&kw| kw == name).unwrap()
    }

    #[test]
    fn test_python_keyword_table_and_mph() {
        assert_eq!(KEYWORD_COUNT, 37);
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
            "false",
            "none",
            "true",
            "AND",
            "AS",
            "asserts",
            "asyncs",
            "awaiting",
            "breaks",
            "cases",
            "classes",
            "continues",
            "defs",
            "dels",
            "elifs",
            "elses",
            "excepts",
            "finallys",
            "fors",
            "froms",
            "globals",
            "ifs",
            "imports",
            "ins",
            "iss",
            "lambdas",
            "matches",
            "nonlocals",
            "nots",
            "ors",
            "passes",
            "raises",
            "returns",
            "trys",
            "whiles",
            "withs",
            "yields",
            "self",
            "cls",
            "fn",
            "function",
            "let",
            "var",
            "const",
            "struct",
            "enum",
            "null",
            "nil",
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
    fn test_python_extensions() {
        assert!(is_python_extension(b"script.py"));
        assert!(is_python_extension(b"stub.pyi"));
        assert!(is_python_extension(b"gui.pyw"));
        assert!(is_python_extension(b"UPPER.PY"));
        assert!(is_python_extension(b"UPPER.PYI"));
        assert!(is_python_extension(b"UPPER.PYW"));
        assert!(is_python_extension(b"a/b/c/foo.bar.py"));

        assert!(!is_python_extension(b"main.c"));
        assert!(!is_python_extension(b"main.cpp"));
        assert!(!is_python_extension(b"main.rs"));
        assert!(!is_python_extension(b"script.pyc"));
        assert!(!is_python_extension(b"script.pyd"));
        assert!(!is_python_extension(b"script.pyo"));
        assert!(!is_python_extension(b"py"));
        assert!(!is_python_extension(b""));
    }

    #[test]
    fn test_python_comments_and_strings() {
        let src = concat!(
            "# comment with def class if else while for\n",
            "def foo(x):\n",
            "    # inline comment with return yield\n",
            "    if x:\n",
            "        s1 = 'single quoted if while'\n",
            "        s2 = \"double quoted def return\"\n",
            "        s3 = 'escaped \\'quote\\' while'\n",
            "        s4 = \"escaped \\\"quote\\\" for\"\n",
            "    return None\n",
            "\n",
            "'''triple single string\n",
            "with class and def inside\n",
            "'''\n",
            "\"\"\"triple double string\n",
            "with while and try inside\n",
            "\"\"\"\n",
            "match x:\n",
            "    case 1:\n",
            "        pass\n"
        );

        let mut res = ParseResult::default();
        parse_into(src.as_bytes(), &mut res);

        assert_eq!(res.keyword_counts[find_kw_id("def")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("if")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("return")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("None")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("match")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("case")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("pass")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("class")], 0);
        assert_eq!(res.keyword_counts[find_kw_id("while")], 0);
        assert_eq!(res.keyword_counts[find_kw_id("for")], 0);
        assert_eq!(res.keyword_counts[find_kw_id("yield")], 0);
        assert_eq!(res.keyword_counts[find_kw_id("try")], 0);
        assert_eq!(res.total_keywords, 7);
        assert_eq!(res.lines_processed, 19);
    }

    #[test]
    fn test_python_string_prefixes() {
        let src = concat!(
            "r1 = r'raw single string with if return'\n",
            "r2 = r\"raw double string with while for\"\n",
            "f1 = f'f-string with def class'\n",
            "f2 = f\"f-string with async await\"\n",
            "b1 = b'byte string with try except'\n",
            "b2 = b\"byte string with import from\"\n",
            "rf1 = rf'''multiline raw f-string\n",
            "with nonlocal global inside\n",
            "'''\n",
            "fr2 = fr\"\"\"multiline f-raw string\n",
            "with lambda yield inside\n",
            "\"\"\"\n",
            "if r1:\n",
            "    return True\n"
        );

        let mut res = ParseResult::default();
        parse_into(src.as_bytes(), &mut res);

        assert_eq!(res.keyword_counts[find_kw_id("if")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("return")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("True")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("def")], 0);
        assert_eq!(res.keyword_counts[find_kw_id("class")], 0);
        assert_eq!(res.keyword_counts[find_kw_id("while")], 0);
        assert_eq!(res.keyword_counts[find_kw_id("for")], 0);
        assert_eq!(res.keyword_counts[find_kw_id("async")], 0);
        assert_eq!(res.keyword_counts[find_kw_id("await")], 0);
        assert_eq!(res.keyword_counts[find_kw_id("try")], 0);
        assert_eq!(res.keyword_counts[find_kw_id("except")], 0);
        assert_eq!(res.keyword_counts[find_kw_id("import")], 0);
        assert_eq!(res.keyword_counts[find_kw_id("from")], 0);
        assert_eq!(res.keyword_counts[find_kw_id("nonlocal")], 0);
        assert_eq!(res.keyword_counts[find_kw_id("global")], 0);
        assert_eq!(res.keyword_counts[find_kw_id("lambda")], 0);
        assert_eq!(res.keyword_counts[find_kw_id("yield")], 0);
        assert_eq!(res.total_keywords, 3);
        assert_eq!(res.lines_processed, 14);
    }

    #[test]
    fn test_python_avx2_boundaries() {
        let len = 32 * 300 + 17;
        let mut buf = vec![b' '; len];
        let kw = b"continue";
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
        assert_eq!(res.keyword_counts[find_kw_id("continue")], placed);

        for start_off in 0..96 {
            for &kw_str in &["as", "def", "elif", "while", "continue"] {
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
    fn test_python_malformed_and_eof_edge_cases() {
        for empty_or_tiny in [
            "",
            "#",
            "# no newline",
            "\"",
            "'",
            "\"\"\"",
            "'''",
            "\"unclosed string if return",
            "'unclosed single if return",
            "\"\"\"unclosed triple double if return",
            "'''unclosed triple single if return",
            "r\"",
            "f'",
            "rf\"\"\"",
        ] {
            let mut res = ParseResult::default();
            parse_into(empty_or_tiny.as_bytes(), &mut res);
            assert_eq!(res.total_keywords, 0);
        }

        let tricky = concat!(
            "def # comment\n",
            "'''triple''' return \"\"\"triple2\"\"\" while # end"
        );
        let mut res = ParseResult::default();
        parse_into(tricky.as_bytes(), &mut res);
        assert_eq!(res.total_keywords, 3);
        assert_eq!(res.keyword_counts[find_kw_id("def")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("return")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("while")], 1);
        assert_eq!(res.lines_processed, 2);
    }

    #[test]
    fn test_python_lines_code_comments_blanks() {
        let src = concat!(
            "# comment\n",
            "\n",
            "def foo():\n",
            "    \"\"\"docstring\n",
            "    line 2\n",
            "    \"\"\"\n",
            "    x = 10  # inline\n",
            "    return x\n"
        );
        let mut res = ParseResult::default();
        parse_into(src.as_bytes(), &mut res);
        assert_eq!(res.lines_processed, 8);
        assert_eq!(res.blank_lines, 1);
        assert_eq!(res.comment_lines, 1);
        assert_eq!(res.code_lines, 6);
    }
}
