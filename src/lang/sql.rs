use super::{
    compute_keyword_lens, compute_kw_max_len, compute_kw_min_len, count_newlines_avx2,
    finish_line_comment, handle_c_block_comment, handle_quoted_literal, kw_hash_fml_add,
    make_kw_table_fml_add, make_token_kind, make_token_lut_lo, match_keyword_caseless_with,
    unified_parse_into, LangLexer, ParseResult, STATE_CODE,
};

pub const KEYWORD_COUNT: usize = 100;
const KW_SLOT_MASK: usize = 511;

pub const KEYWORD_NAMES: [&str; KEYWORD_COUNT] = [
    "all",
    "alter",
    "and",
    "any",
    "as",
    "asc",
    "begin",
    "between",
    "by",
    "cascade",
    "case",
    "cast",
    "check",
    "coalesce",
    "collate",
    "commit",
    "constraint",
    "create",
    "cross",
    "database",
    "declare",
    "default",
    "delete",
    "desc",
    "distinct",
    "drop",
    "else",
    "end",
    "except",
    "exec",
    "execute",
    "exists",
    "false",
    "fetch",
    "first",
    "for",
    "foreign",
    "from",
    "full",
    "function",
    "grant",
    "group",
    "having",
    "if",
    "in",
    "index",
    "inner",
    "insert",
    "intersect",
    "into",
    "is",
    "join",
    "key",
    "left",
    "like",
    "limit",
    "natural",
    "next",
    "not",
    "null",
    "offset",
    "on",
    "only",
    "or",
    "order",
    "outer",
    "over",
    "partition",
    "primary",
    "procedure",
    "recursive",
    "references",
    "replace",
    "return",
    "returns",
    "revoke",
    "right",
    "rollback",
    "row",
    "rows",
    "schema",
    "select",
    "set",
    "some",
    "table",
    "then",
    "to",
    "transaction",
    "trigger",
    "true",
    "truncate",
    "union",
    "unique",
    "update",
    "values",
    "view",
    "when",
    "where",
    "window",
    "with",
];

pub const KEYWORD_LENS: [u8; KEYWORD_COUNT] = compute_keyword_lens(&KEYWORD_NAMES);
pub const KW_MIN_LEN: usize = compute_kw_min_len(&KEYWORD_NAMES);
pub const KW_MAX_LEN: usize = compute_kw_max_len(&KEYWORD_NAMES);

const TOK_IDENT: u8 = 3;
const TOK_MINUS: u8 = 4;
const TOK_SLASH: u8 = 5;
const TOK_QUOTE: u8 = 6;
const TOK_APOSTROPHE: u8 = 7;
const TOK_BACKTICK: u8 = 8;
const TOK_HASH: u8 = 9;

#[inline(always)]
const fn kw_hash(first: u8, mid: u8, last: u8, len: usize) -> usize {
    kw_hash_fml_add(first, mid, last, len, [1, 61, 63, 99], 1, KW_SLOT_MASK)
}

const KW_TABLE: [u8; 512] = make_kw_table_fml_add(&KEYWORD_NAMES, [1, 61, 63, 99], 1);

pub const TOKEN_KIND: [u8; 256] = make_token_kind(
    TOK_IDENT,
    &[
        (b'-', TOK_MINUS),
        (b'/', TOK_SLASH),
        (b'"', TOK_QUOTE),
        (b'\'', TOK_APOSTROPHE),
        (b'`', TOK_BACKTICK),
        (b'#', TOK_HASH),
    ],
);

pub const TOKEN_LUT_LO: [i8; 32] = make_token_lut_lo(&TOKEN_KIND);

#[cfg(test)]
#[inline(always)]
pub fn is_sql_extension(name: &[u8]) -> bool {
    super::detect_language(name, super::LANG_ALL_MASK) == Some(super::LangId::Sql)
}

pub struct SqlLexer;

impl LangLexer<KEYWORD_COUNT> for SqlLexer {
    const TOKEN_KIND: [u8; 256] = TOKEN_KIND;
    const TOKEN_LUT_LO: [i8; 32] = TOKEN_LUT_LO;
    const HAS_DOLLAR_IDENT: bool = false;
    const SUPPORTS_TEMPLATES: bool = false;

    #[inline(always)]
    fn match_keyword(cand: &[u8]) -> Option<usize> {
        match_keyword_caseless_with(
            cand,
            KW_MIN_LEN,
            KW_MAX_LEN,
            &KW_TABLE,
            &KEYWORD_LENS,
            &KEYWORD_NAMES,
            |c| {
                kw_hash(
                    c[0].to_ascii_lowercase(),
                    c[c.len() / 2].to_ascii_lowercase(),
                    c[c.len() - 1].to_ascii_lowercase(),
                    c.len(),
                )
            },
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
                    finish_line_comment(content, i + 2, line_state, acc)
                } else {
                    *line_state |= STATE_CODE;
                    i + 1
                }
            }
            TOK_SLASH => {
                if i + 1 < content_len && content[i + 1] == b'*' {
                    handle_c_block_comment(content, i + 2, line_state, acc)
                } else {
                    *line_state |= STATE_CODE;
                    i + 1
                }
            }
            TOK_HASH => finish_line_comment(content, i + 1, line_state, acc),
            TOK_APOSTROPHE => {
                *line_state |= STATE_CODE;
                let mut cur = i + 1;
                while cur < content_len {
                    if content[cur] == b'\'' {
                        if cur + 1 < content_len && content[cur + 1] == b'\'' {
                            cur += 2;
                            continue;
                        }
                        cur += 1;
                        break;
                    }
                    if content[cur] == b'\\' && cur + 1 < content_len {
                        cur += 2;
                        continue;
                    }
                    cur += 1;
                }
                let nl = count_newlines_avx2(&content[i + 1..cur]);
                if nl > 0 {
                    acc.code_lines += nl as i64;
                    acc.lines_processed += nl as i64;
                }
                cur
            }
            TOK_QUOTE => handle_quoted_literal(content, i, b'"', line_state, acc),
            TOK_BACKTICK => handle_quoted_literal(content, i, b'`', line_state, acc),
            _ => {
                *line_state |= STATE_CODE;
                i + 1
            }
        }
    }
}

pub fn parse_into(content: &[u8], acc: &mut ParseResult<KEYWORD_COUNT>) {
    unified_parse_into::<KEYWORD_COUNT, SqlLexer>(content, acc);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sql_extensions() {
        assert!(is_sql_extension(b"query.sql"));
        assert!(!is_sql_extension(b"query.sq"));
    }

    #[test]
    fn test_sql_keyword_table_and_mph() {
        assert_eq!(KEYWORD_NAMES.len(), KEYWORD_COUNT);
        assert_eq!(KEYWORD_LENS.len(), KEYWORD_COUNT);
        for (i, name) in KEYWORD_NAMES.iter().enumerate() {
            let b = name.as_bytes();
            let h = kw_hash(b[0], b[b.len() / 2], b[b.len() - 1], b.len());
            assert_eq!(KW_TABLE[h], i as u8);
        }
    }

    #[test]
    fn test_sql_comments_and_strings() {
        let code = b"-- line comment\nSELECT * FROM users /* inline comment */ WHERE name = 'O''Reilly';\n/* multiline\ncomment */\nINSERT INTO t VALUES (1);\n";
        let mut acc = ParseResult::<KEYWORD_COUNT>::default();
        parse_into(code, &mut acc);
        assert_eq!(acc.comment_lines, 3);
        assert_eq!(acc.code_lines, 2);
        assert_eq!(acc.lines_processed, 5);
        assert_eq!(acc.total_keywords, 6);
    }
}
