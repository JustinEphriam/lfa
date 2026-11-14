use super::{
    compute_keyword_lens, compute_kw_max_len, compute_kw_min_len, count_newlines_avx2,
    find_byte_avx2, finish_block_comment, handle_nested_c_slash_comments, handle_quoted_literal,
    kw_hash_fsl_add, make_kw_table_fsl_add, make_token_kind, make_token_lut_lo, match_keyword_with,
    skip_nested_plus_slash_comment, unified_parse_into, LangLexer, ParseResult, STATE_CODE,
};

pub const KEYWORD_COUNT: usize = 99;
const KW_SLOT_MASK: usize = 511;

pub const KEYWORD_NAMES: [&str; KEYWORD_COUNT] = [
    "abstract",
    "alias",
    "align",
    "asm",
    "assert",
    "auto",
    "body",
    "bool",
    "byte",
    "case",
    "cast",
    "catch",
    "cdouble",
    "cent",
    "cfloat",
    "char",
    "class",
    "const",
    "continue",
    "creal",
    "dchar",
    "debug",
    "default",
    "delegate",
    "delete",
    "deprecated",
    "do",
    "double",
    "else",
    "enum",
    "export",
    "extern",
    "false",
    "final",
    "finally",
    "float",
    "for",
    "foreach",
    "foreach_reverse",
    "function",
    "goto",
    "idouble",
    "if",
    "ifloat",
    "immutable",
    "import",
    "in",
    "inout",
    "int",
    "interface",
    "invariant",
    "ireal",
    "is",
    "lazy",
    "long",
    "macro",
    "mixin",
    "module",
    "new",
    "nothrow",
    "null",
    "out",
    "override",
    "package",
    "pragma",
    "private",
    "protected",
    "public",
    "pure",
    "real",
    "ref",
    "return",
    "scope",
    "shared",
    "short",
    "static",
    "struct",
    "super",
    "switch",
    "synchronized",
    "template",
    "this",
    "throw",
    "true",
    "try",
    "typeid",
    "typeof",
    "ubyte",
    "ucent",
    "uint",
    "ulong",
    "union",
    "unittest",
    "ushort",
    "version",
    "void",
    "wchar",
    "while",
    "with",
];

pub const KEYWORD_LENS: [u8; KEYWORD_COUNT] = compute_keyword_lens(&KEYWORD_NAMES);
pub const KW_MIN_LEN: usize = compute_kw_min_len(&KEYWORD_NAMES);
pub const KW_MAX_LEN: usize = compute_kw_max_len(&KEYWORD_NAMES);

#[inline(always)]
const fn kw_hash(first: u8, second: u8, last: u8, len: usize) -> usize {
    kw_hash_fsl_add(first, second, last, len, [33, 205, 10, 230], 1, KW_SLOT_MASK)
}

const KW_TABLE: [u8; 512] = make_kw_table_fsl_add(&KEYWORD_NAMES, [33, 205, 10, 230], 1);

const TOK_IDENT: u8 = 3;
const TOK_SLASH: u8 = 4;
const TOK_QUOTE: u8 = 5;
const TOK_APOSTROPHE: u8 = 6;
const TOK_BACKTICK: u8 = 7;

pub const TOKEN_KIND: [u8; 256] = make_token_kind(
    TOK_IDENT,
    &[
        (b'/', TOK_SLASH),
        (b'"', TOK_QUOTE),
        (b'\'', TOK_APOSTROPHE),
        (b'`', TOK_BACKTICK),
    ],
);

pub const TOKEN_LUT_LO: [i8; 32] = make_token_lut_lo(&TOKEN_KIND);

pub fn is_d_dep_file(probe: &[u8]) -> bool {
    let mut i = 0;
    while i < probe.len() && probe[i] != b'\n' {
        i += 1;
    }
    let first_line = &probe[..i];
    let mut has_colon_space = false;
    let mut j = 0;
    while j + 1 < first_line.len() {
        if first_line[j] == b':' && (first_line[j + 1] == b' ' || first_line[j + 1] == b'\t') {
            has_colon_space = true;
            break;
        }
        j += 1;
    }
    if !has_colon_space {
        return false;
    }
    let target_part = &first_line[..j];
    !target_part.is_empty() && !target_part.contains(&b' ') && !target_part.contains(&b'\t')
}

#[cfg(test)]
#[inline(always)]
pub fn is_d_extension(name: &[u8]) -> bool {
    super::detect_language(name, super::LANG_ALL_MASK) == Some(super::LangId::D)
}

pub struct DLexer;

impl LangLexer<KEYWORD_COUNT> for DLexer {
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
            |c| kw_hash(c[0], c[1], c[c.len() - 1], c.len()),
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
            TOK_SLASH => {
                if i + 1 < content_len && content[i + 1] == b'+' {
                    let end_i = skip_nested_plus_slash_comment(content, i + 2);
                    return finish_block_comment(content, i + 2, end_i, line_state, acc);
                }
                handle_nested_c_slash_comments(content, i, line_state, acc)
            }
            TOK_QUOTE => handle_quoted_literal(content, i, b'"', line_state, acc),
            TOK_APOSTROPHE => handle_quoted_literal(content, i, b'\'', line_state, acc),
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

pub fn parse_into(content: &[u8], acc: &mut ParseResult<KEYWORD_COUNT>) {
    unified_parse_into::<KEYWORD_COUNT, DLexer>(content, acc);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_d_extensions() {
        assert!(is_d_extension(b"main.d"));
        assert!(is_d_extension(b"interface.di"));
        assert!(!is_d_extension(b"main.rs"));
        assert!(!is_d_extension(b"main.c"));
    }

    #[test]
    fn test_d_keyword_table_and_mph() {
        for (i, name) in KEYWORD_NAMES.iter().enumerate() {
            let b = name.as_bytes();
            let second = if b.len() > 1 { b[1] } else { b' ' };
            let h = kw_hash(b[0], second, b[b.len() - 1], b.len());
            assert_eq!(KW_TABLE[h], i as u8);
            assert_eq!(DLexer::match_keyword(b), Some(i));
        }
        assert_eq!(DLexer::match_keyword(b"notakeyword"), None);
        assert_eq!(DLexer::match_keyword(b"d"), None);
    }

    #[test]
    fn test_d_comments_and_strings() {
        let src = b"// comment\nimport std.stdio;\n/+ nested /+ comment +/ here +/\nvoid main() {\n    int x = 42;\n    string s = `wysiwyg\nstring`;\n    writeln(\"hello\");\n}\n";
        let mut acc = ParseResult::default();
        parse_into(src, &mut acc);
        assert_eq!(acc.comment_lines, 2);
        assert_eq!(acc.code_lines, 7);
        assert_eq!(acc.total_keywords, 3);
    }

    #[test]
    fn test_d_dep_file_detection() {
        assert!(is_d_dep_file(b"/path/to/foo.d: src/main.rs\n"));
        assert!(is_d_dep_file(b"target/debug/foo.o: foo.c\n"));
        assert!(is_d_dep_file(b"foo.rlib: bar.rs\n"));
        assert!(is_d_dep_file(b"libfoo.a: foo.o bar.o\n"));
        assert!(!is_d_dep_file(b"import std.stdio;\nvoid main() {}\n"));
        assert!(!is_d_dep_file(b"module test.d;\n"));
        assert!(!is_d_dep_file(b"// a comment\nint x = 10;\n"));
    }
}
