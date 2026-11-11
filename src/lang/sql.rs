use super::{
    compute_keyword_lens, compute_kw_max_len, compute_kw_min_len, count_newlines_avx2,
    find_byte_avx2, make_token_lut_lo, skip_literal, unified_parse_into, LangLexer,
    ParseResult, STATE_CODE, STATE_COMMENT, TOK_CODE, TOK_NL, TOK_WS,
};

pub const KEYWORD_COUNT: usize = 100;
const KW_SLOT_MASK: usize = 511;
const KW_EMPTY: u8 = 255;

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

const _: () = assert!(KEYWORD_NAMES.len() == KEYWORD_COUNT);
const _: () = assert!(KEYWORD_LENS.len() == KEYWORD_COUNT);

const TOK_IDENT: u8 = 3;
const TOK_MINUS: u8 = 4;
const TOK_SLASH: u8 = 5;
const TOK_QUOTE: u8 = 6;
const TOK_APOSTROPHE: u8 = 7;
const TOK_BACKTICK: u8 = 8;
const TOK_HASH: u8 = 9;

#[inline(always)]
const fn kw_hash(first: u8, mid: u8, last: u8, len: usize) -> usize {
    let v = (first as usize)
        .wrapping_mul(1)
        .wrapping_add((mid as usize).wrapping_mul(61))
        .wrapping_add((last as usize).wrapping_mul(63))
        .wrapping_add(len.wrapping_mul(99));
    (v ^ (v >> 1)) & KW_SLOT_MASK
}

const KW_TABLE: [u8; 512] = {
    let mut table = [KW_EMPTY; 512];
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

#[inline(always)]
fn kw_match_case_insensitive(cand: &[u8], target: &[u8]) -> bool {
    if cand.len() != target.len() {
        return false;
    }
    for i in 0..cand.len() {
        let b = cand[i];
        let lower = if b.is_ascii_uppercase() { b + 32 } else { b };
        if lower != target[i] {
            return false;
        }
    }
    true
}

pub const TOKEN_KIND: [u8; 256] = {
    let mut table = [TOK_CODE; 256];
    table[b' ' as usize] = TOK_WS;
    table[b'\t' as usize] = TOK_WS;
    table[b'\r' as usize] = TOK_WS;
    table[0x0b_usize] = TOK_WS;
    table[0x0c_usize] = TOK_WS;
    table[b'\n' as usize] = TOK_NL;
    let mut c = b'a';
    while c <= b'z' {
        table[c as usize] = TOK_IDENT;
        c += 1;
    }
    let mut c = b'A';
    while c <= b'Z' {
        table[c as usize] = TOK_IDENT;
        c += 1;
    }
    table[b'_' as usize] = TOK_IDENT;
    table[b'-' as usize] = TOK_MINUS;
    table[b'/' as usize] = TOK_SLASH;
    table[b'"' as usize] = TOK_QUOTE;
    table[b'\'' as usize] = TOK_APOSTROPHE;
    table[b'`' as usize] = TOK_BACKTICK;
    table[b'#' as usize] = TOK_HASH;
    table
};

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
    const HAS_SPECIAL_IDENT: bool = true;
    const HAS_LINE_START_HOOK: bool = false;

    #[inline(always)]
    fn match_keyword(_cand: &[u8]) -> Option<usize> {
        None
    }

    #[inline(always)]
    fn handle_ident(
        content: &[u8],
        start: usize,
        end: usize,
        _line_state: &mut u8,
        acc: &mut ParseResult<KEYWORD_COUNT>,
    ) -> usize {
        let cand = &content[start..end];
        let cand_len = cand.len();
        if (KW_MIN_LEN..=KW_MAX_LEN).contains(&cand_len) {
            let first = if cand[0].is_ascii_uppercase() {
                cand[0] + 32
            } else {
                cand[0]
            };
            let mid_b = cand[cand_len / 2];
            let mid = if mid_b.is_ascii_uppercase() {
                mid_b + 32
            } else {
                mid_b
            };
            let last_b = cand[cand_len - 1];
            let last = if last_b.is_ascii_uppercase() {
                last_b + 32
            } else {
                last_b
            };

            let h = kw_hash(first, mid, last, cand_len);
            let idx = KW_TABLE[h] as usize;
            if idx < KEYWORD_COUNT && kw_match_case_insensitive(cand, KEYWORD_NAMES[idx].as_bytes()) {
                acc.keyword_counts[idx] += 1;
                acc.total_keywords += 1;
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
        let content_len = content.len();
        match kind {
            TOK_MINUS => {
                if i + 1 < content_len && content[i + 1] == b'-' {
                    *line_state |= STATE_COMMENT;
                    if let Some(pos) = find_byte_avx2(&content[i + 2..], b'\n') {
                        if (*line_state & STATE_CODE) != 0 {
                            acc.code_lines += 1;
                        } else {
                            acc.comment_lines += 1;
                        }
                        acc.lines_processed += 1;
                        *line_state = 0;
                        i + 2 + pos + 1
                    } else {
                        content_len
                    }
                } else {
                    *line_state |= STATE_CODE;
                    i + 1
                }
            }
            TOK_SLASH => {
                if i + 1 < content_len && content[i + 1] == b'*' {
                    *line_state |= STATE_COMMENT;
                    let mut cur = i + 2;
                    while cur + 1 < content_len {
                        if content[cur] == b'*' && content[cur + 1] == b'/' {
                            return cur + 2;
                        }
                        if content[cur] == b'\n' {
                            if (*line_state & STATE_CODE) != 0 {
                                acc.code_lines += 1;
                            } else {
                                acc.comment_lines += 1;
                            }
                            acc.lines_processed += 1;
                            *line_state = STATE_COMMENT;
                        }
                        cur += 1;
                    }
                    while cur < content_len {
                        if content[cur] == b'\n' {
                            if (*line_state & STATE_CODE) != 0 {
                                acc.code_lines += 1;
                            } else {
                                acc.comment_lines += 1;
                            }
                            acc.lines_processed += 1;
                            *line_state = STATE_COMMENT;
                        }
                        cur += 1;
                    }
                    content_len
                } else {
                    *line_state |= STATE_CODE;
                    i + 1
                }
            }
            TOK_HASH => {
                *line_state |= STATE_COMMENT;
                if let Some(pos) = find_byte_avx2(&content[i + 1..], b'\n') {
                    if (*line_state & STATE_CODE) != 0 {
                        acc.code_lines += 1;
                    } else {
                        acc.comment_lines += 1;
                    }
                    acc.lines_processed += 1;
                    *line_state = 0;
                    i + 1 + pos + 1
                } else {
                    content_len
                }
            }
            TOK_APOSTROPHE => {
                *line_state |= STATE_CODE;
                let mut cur = i + 1;
                while cur < content_len {
                    if content[cur] == b'\'' {
                        if cur + 1 < content_len && content[cur + 1] == b'\'' {
                            cur += 2;
                            continue;
                        }
                        return cur + 1;
                    }
                    if content[cur] == b'\\' && cur + 1 < content_len {
                        cur += 2;
                        continue;
                    }
                    if content[cur] == b'\n' {
                        acc.code_lines += 1;
                        acc.lines_processed += 1;
                    }
                    cur += 1;
                }
                content_len
            }
            TOK_QUOTE => {
                *line_state |= STATE_CODE;
                let end_i = skip_literal(content, i + 1, b'"');
                let nl = count_newlines_avx2(&content[i + 1..end_i]);
                if nl > 0 {
                    acc.code_lines += nl as i64;
                    acc.lines_processed += nl as i64;
                }
                end_i
            }
            TOK_BACKTICK => {
                *line_state |= STATE_CODE;
                let end_i = skip_literal(content, i + 1, b'`');
                let nl = count_newlines_avx2(&content[i + 1..end_i]);
                if nl > 0 {
                    acc.code_lines += nl as i64;
                    acc.lines_processed += nl as i64;
                }
                end_i
            }
            _ => i + 1,
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
