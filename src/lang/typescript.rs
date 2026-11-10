use super::{
    compute_keyword_lens, compute_kw_max_len, compute_kw_min_len, count_newlines_avx2,
    find_byte_avx2, find_star_slash, kw_match_2_to_16, make_token_lut_lo,
    skip_literal, unified_parse_into, LangLexer, ParseResult, STATE_CODE, STATE_COMMENT,
    TOK_CODE, TOK_NL, TOK_WS,
};

pub const KEYWORD_COUNT: usize = 75;
const KW_SLOT_MASK: usize = 255;
const KW_EMPTY: u8 = 255;

pub const KEYWORD_NAMES: [&str; KEYWORD_COUNT] = [
    "abstract",
    "any",
    "as",
    "asserts",
    "async",
    "await",
    "bigint",
    "boolean",
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "debugger",
    "declare",
    "default",
    "delete",
    "do",
    "else",
    "enum",
    "export",
    "extends",
    "false",
    "finally",
    "for",
    "function",
    "get",
    "global",
    "if",
    "implements",
    "import",
    "in",
    "infer",
    "instanceof",
    "interface",
    "intrinsic",
    "is",
    "keyof",
    "let",
    "module",
    "namespace",
    "never",
    "new",
    "null",
    "number",
    "object",
    "override",
    "package",
    "private",
    "protected",
    "public",
    "readonly",
    "return",
    "satisfies",
    "set",
    "static",
    "string",
    "super",
    "switch",
    "symbol",
    "this",
    "throw",
    "true",
    "try",
    "type",
    "typeof",
    "undefined",
    "unique",
    "unknown",
    "var",
    "void",
    "while",
    "with",
    "yield",
];

pub const KEYWORD_LENS: [u8; KEYWORD_COUNT] = compute_keyword_lens(&KEYWORD_NAMES);
pub const KW_MIN_LEN: usize = compute_kw_min_len(&KEYWORD_NAMES);
pub const KW_MAX_LEN: usize = compute_kw_max_len(&KEYWORD_NAMES);

const _: () = assert!(KEYWORD_NAMES.len() == KEYWORD_COUNT);
const _: () = assert!(KEYWORD_LENS.len() == KEYWORD_COUNT);

const TOK_IDENT: u8 = 3;
const TOK_SLASH: u8 = 4;
const TOK_QUOTE: u8 = 5;
const TOK_APOSTROPHE: u8 = 6;
const TOK_BACKTICK: u8 = 7;


#[inline(always)]
const fn kw_hash(first: u8, mid: u8, last: u8, len: usize) -> usize {
    let val = (first as usize)
        .wrapping_mul(2339)
        .wrapping_add((mid as usize).wrapping_mul(1343))
        .wrapping_add((last as usize).wrapping_mul(2946))
        .wrapping_add(len.wrapping_mul(2860));
    (val ^ (val >> 5)) & KW_SLOT_MASK
}

const KW_TABLE: [u8; 256] = {
    let mut table = [KW_EMPTY; 256];
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
    table[b'$' as usize] = TOK_IDENT;
    table[b'/' as usize] = TOK_SLASH;
    table[b'"' as usize] = TOK_QUOTE;
    table[b'\'' as usize] = TOK_APOSTROPHE;
    table[b'`' as usize] = TOK_BACKTICK;
    table
};

pub const TOKEN_LUT_LO: [i8; 32] = make_token_lut_lo(&TOKEN_KIND);

pub const TOKEN_KIND_INTERP: [u8; 256] = {
    let mut table = TOKEN_KIND;
    table[b'{' as usize] = 8;
    table[b'}' as usize] = 9;
    table
};

pub const TOKEN_INTERP_LUT_LO: [i8; 32] = make_token_lut_lo(&TOKEN_KIND_INTERP);

#[cfg(test)]
#[inline(always)]
pub fn is_typescript_extension(name: &[u8]) -> bool {
    super::detect_language(name, super::LANG_ALL_MASK) == Some(super::LangId::TypeScript)
}

pub struct TsLexer;

impl LangLexer<KEYWORD_COUNT> for TsLexer {
    const TOKEN_KIND: [u8; 256] = TOKEN_KIND;
    const TOKEN_LUT_LO: [i8; 32] = TOKEN_LUT_LO;
    const HAS_DOLLAR_IDENT: bool = true;
    const SUPPORTS_TEMPLATES: bool = true;
    const TOKEN_KIND_INTERP: [u8; 256] = TOKEN_KIND_INTERP;
    const TOKEN_INTERP_LUT_LO: [i8; 32] = TOKEN_INTERP_LUT_LO;

    #[inline(always)]
    fn match_keyword(cand: &[u8]) -> Option<usize> {
        let n = cand.len();
        if n.wrapping_sub(KW_MIN_LEN) <= KW_MAX_LEN - KW_MIN_LEN {
            let h = kw_hash(cand[0], cand[n / 2], cand[n - 1], n);
            let slot = KW_TABLE[h];
            if slot != KW_EMPTY {
                let idx = slot as usize;
                if n == KEYWORD_LENS[idx] as usize
                    && kw_match_2_to_16(cand, KEYWORD_NAMES[idx].as_bytes())
                {
                    return Some(idx);
                }
            }
        }
        None
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
        let b = content[i];
        match kind {
            TOK_SLASH => {
                if i + 1 < content_len {
                    let next_b = content[i + 1];
                    if next_b == b'/' {
                        *line_state |= STATE_COMMENT;
                        let Some(pos) = find_byte_avx2(&content[i + 2..], b'\n') else {
                            return content_len;
                        };
                        let nl_idx = i + 2 + pos;
                        if (*line_state & STATE_CODE) != 0 {
                            acc.code_lines += 1;
                        } else if (*line_state & STATE_COMMENT) != 0 {
                            acc.comment_lines += 1;
                        } else {
                            acc.blank_lines += 1;
                        }
                        acc.lines_processed += 1;
                        *line_state = 0;
                        return nl_idx + 1;
                    } else if next_b == b'*' {
                        *line_state |= STATE_COMMENT;
                        if let Some(pos) = find_star_slash(&content[i + 2..]) {
                            let star_slash_pos = i + 2 + pos;
                            if let Some(first_nl) =
                                find_byte_avx2(&content[i + 2..star_slash_pos], b'\n')
                            {
                                if (*line_state & STATE_CODE) != 0 {
                                    acc.code_lines += 1;
                                } else {
                                    acc.comment_lines += 1;
                                }
                                acc.lines_processed += 1;
                                let first_nl_abs = i + 2 + first_nl;
                                let inter_nls = count_newlines_avx2(
                                    &content[first_nl_abs + 1..star_slash_pos],
                                );
                                acc.comment_lines += inter_nls as i64;
                                acc.lines_processed += inter_nls as i64;
                                *line_state = STATE_COMMENT;
                            }
                            return star_slash_pos + 2;
                        } else {
                            if let Some(first_nl) = find_byte_avx2(&content[i + 2..], b'\n') {
                                if (*line_state & STATE_CODE) != 0 {
                                    acc.code_lines += 1;
                                } else {
                                    acc.comment_lines += 1;
                                }
                                acc.lines_processed += 1;
                                let first_nl_abs = i + 2 + first_nl;
                                let rem_nls = count_newlines_avx2(&content[first_nl_abs + 1..]);
                                acc.comment_lines += rem_nls as i64;
                                acc.lines_processed += rem_nls as i64;
                                *line_state = STATE_COMMENT;
                            }
                            return content_len;
                        }
                    }
                }
                *line_state |= STATE_CODE;
                i + 1
            }
            TOK_QUOTE | TOK_APOSTROPHE => {
                *line_state |= STATE_CODE;
                let end_i = skip_literal(content, i + 1, b);
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
    unified_parse_into::<KEYWORD_COUNT, TsLexer>(content, acc);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn find_kw_id(name: &str) -> usize {
        KEYWORD_NAMES.iter().position(|&kw| kw == name).unwrap()
    }

    #[test]
    fn test_ts_keyword_table_and_mph() {
        assert_eq!(KEYWORD_COUNT, 75);
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
            assert_eq!(KW_TABLE[h], i as u8);

            let mut res = ParseResult::default();
            parse_into(bytes, &mut res);
            assert_eq!(res.total_keywords, 1);
            assert_eq!(res.keyword_counts[i], 1);
        }

        let non_keywords = ["console", "Record", "Promise", "Partial", "types", "interfaces"];
        for &nk in &non_keywords {
            let mut res = ParseResult::default();
            parse_into(nk.as_bytes(), &mut res);
            assert_eq!(res.total_keywords, 0);
        }
    }

    #[test]
    fn test_ts_extensions() {
        assert!(is_typescript_extension(b"index.ts"));
        assert!(is_typescript_extension(b"service.mts"));
        assert!(is_typescript_extension(b"common.cts"));
        assert!(is_typescript_extension(b"Component.tsx"));
        assert!(is_typescript_extension(b"UPPER.TS"));
        assert!(!is_typescript_extension(b"ts"));
        assert!(!is_typescript_extension(b".ts"));
        assert!(!is_typescript_extension(b"app.js"));
    }

    #[test]
    fn test_ts_types_and_keywords() {
        let src = b"export interface User {\n    readonly id: number;\n    name: string;\n    role: 'admin' | 'user';\n}\n\ntype Action = 'create' | 'delete';\n\ndeclare function handle(u: unknown): any;\n";
        let mut res = ParseResult::default();
        parse_into(src, &mut res);

        assert_eq!(res.keyword_counts[find_kw_id("export")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("interface")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("readonly")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("number")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("string")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("type")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("declare")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("function")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("unknown")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("any")], 1);
        assert_eq!(res.total_keywords, 10);
        assert_eq!(res.lines_processed, 9);
        assert_eq!(res.code_lines, 7);
        assert_eq!(res.blank_lines, 2);
    }

    #[test]
    fn test_ts_comments_and_strings() {
        let src = b"// TS header\n/* multi\n   line\n   comment */\nconst x: string = \"hello // not comment\";\nreturn x;\n";
        let mut res = ParseResult::default();
        parse_into(src, &mut res);

        assert_eq!(res.keyword_counts[find_kw_id("const")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("string")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("return")], 1);
        assert_eq!(res.total_keywords, 3);
        assert_eq!(res.lines_processed, 6);
        assert_eq!(res.code_lines, 2);
        assert_eq!(res.comment_lines, 4);
    }

    #[test]
    fn test_ts_template_literals() {
        let src = b"const query = `\nSELECT * FROM users\nWHERE id = ${userId}\n`;\n";
        let mut res = ParseResult::default();
        parse_into(src, &mut res);

        assert_eq!(res.keyword_counts[find_kw_id("const")], 1);
        assert_eq!(res.total_keywords, 1);
        assert_eq!(res.lines_processed, 4);
        assert_eq!(res.code_lines, 4);

        let interp_src = b"const s: string = `a ${ x ? new Foo<string>() : null } b`;\nif (z) {}\n";
        let mut res2 = ParseResult::default();
        parse_into(interp_src, &mut res2);
        assert_eq!(res2.keyword_counts[find_kw_id("const")], 1);
        assert_eq!(res2.keyword_counts[find_kw_id("string")], 2);
        assert_eq!(res2.keyword_counts[find_kw_id("new")], 1);
        assert_eq!(res2.keyword_counts[find_kw_id("null")], 1);
        assert_eq!(res2.keyword_counts[find_kw_id("if")], 1);
        assert_eq!(res2.total_keywords, 6);
        assert_eq!(res2.lines_processed, 2);
        assert_eq!(res2.code_lines, 2);

        let nested_src = b"let s = `a ${ `in ${ typeof y }` } b`;\nreturn 1;\n";
        let mut res3 = ParseResult::default();
        parse_into(nested_src, &mut res3);
        assert_eq!(res3.keyword_counts[find_kw_id("let")], 1);
        assert_eq!(res3.keyword_counts[find_kw_id("typeof")], 1);
        assert_eq!(res3.keyword_counts[find_kw_id("return")], 1);
        assert_eq!(res3.total_keywords, 3);

        let braces_src = b"`${ (() => { return { a: 1 }; })() }`; while (1) {}\n";
        let mut res4 = ParseResult::default();
        parse_into(braces_src, &mut res4);
        assert_eq!(res4.keyword_counts[find_kw_id("return")], 1);
        assert_eq!(res4.keyword_counts[find_kw_id("while")], 1);
        assert_eq!(res4.total_keywords, 2);

        let escape_src = b"`\\${not_interp} \\` still text`; for (;;) {}\n";
        let mut res5 = ParseResult::default();
        parse_into(escape_src, &mut res5);
        assert_eq!(res5.keyword_counts[find_kw_id("for")], 1);
        assert_eq!(res5.total_keywords, 1);

        let dollar_src = b"`cost: $5 ${ void 0 }`;\n";
        let mut res6 = ParseResult::default();
        parse_into(dollar_src, &mut res6);
        assert_eq!(res6.keyword_counts[find_kw_id("void")], 1);
        assert_eq!(res6.total_keywords, 1);

        let cmt_interp = b"`${ /* if */ x // while\n }`;\n";
        let mut res7 = ParseResult::default();
        parse_into(cmt_interp, &mut res7);
        assert_eq!(res7.keyword_counts[find_kw_id("if")], 0);
        assert_eq!(res7.keyword_counts[find_kw_id("while")], 0);
        assert_eq!(res7.total_keywords, 0);

        let mut pad = vec![b'a'; 40];
        pad.extend_from_slice(b"`${ return }`\n");
        let mut res8 = ParseResult::default();
        parse_into(&pad, &mut res8);
        assert_eq!(res8.keyword_counts[find_kw_id("return")], 1);
        assert_eq!(res8.total_keywords, 1);

        let mut res_unterm = ParseResult::default();
        parse_into(b"`abc ${ if", &mut res_unterm);
        assert_eq!(res_unterm.keyword_counts[find_kw_id("if")], 1);
    }

    #[test]
    fn test_ts_identifiers_with_dollar() {
        let src = b"const $obs: Observable<number> = of(1);\nconst                                    $interface = $obs;\nreturn $interface;\n";
        let mut res = ParseResult::default();
        parse_into(src, &mut res);

        assert_eq!(res.keyword_counts[find_kw_id("const")], 2);
        assert_eq!(res.keyword_counts[find_kw_id("number")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("return")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("interface")], 0);
        assert_eq!(res.total_keywords, 4);
        assert_eq!(res.lines_processed, 3);
        assert_eq!(res.code_lines, 3);
    }

    #[test]
    fn test_ts_avx2_boundaries_and_eof() {
        for len in [31, 32, 33, 63, 64, 65, 127, 128, 129] {
            let mut buf = vec![b' '; len];
            buf.extend_from_slice(b"export async function run(): Promise<boolean> {\n    return true;\n}\n");
            let mut res = ParseResult::default();
            parse_into(&buf, &mut res);
            assert_eq!(res.keyword_counts[find_kw_id("export")], 1);
            assert_eq!(res.keyword_counts[find_kw_id("async")], 1);
            assert_eq!(res.keyword_counts[find_kw_id("function")], 1);
            assert_eq!(res.keyword_counts[find_kw_id("boolean")], 1);
            assert_eq!(res.keyword_counts[find_kw_id("return")], 1);
            assert_eq!(res.keyword_counts[find_kw_id("true")], 1);
            assert_eq!(res.total_keywords, 6);
        }

        let mut res = ParseResult::default();
        parse_into(b"// unclosed line comment at eof", &mut res);
        assert_eq!(res.total_keywords, 0);
        assert_eq!(res.comment_lines, 1);
        assert_eq!(res.lines_processed, 1);
    }
}
