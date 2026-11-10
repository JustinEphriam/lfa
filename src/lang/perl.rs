use super::{
    finish_trailing_line, STATE_CODE, STATE_COMMENT, compute_keyword_lens, compute_kw_max_len, compute_kw_min_len, count_newlines_avx2,
    find_byte_avx2, kw_match_2_to_16, make_token_lut_lo, skip_ident_cont_avx2, skip_literal,
    skip_non_tokens_with_lut_avx2, ParseResult, TOK_CODE, TOK_NL, TOK_WS,
};

pub const KEYWORD_COUNT: usize = 73;
const KW_SLOT_MASK: usize = 255;
const KW_EMPTY: u8 = 255;

pub const KEYWORD_NAMES: [&str; KEYWORD_COUNT] = [
    "if",
    "elsif",
    "else",
    "unless",
    "while",
    "until",
    "for",
    "foreach",
    "do",
    "continue",
    "last",
    "next",
    "redo",
    "goto",
    "return",
    "sub",
    "my",
    "our",
    "state",
    "local",
    "use",
    "no",
    "require",
    "package",
    "eval",
    "die",
    "warn",
    "exit",
    "given",
    "when",
    "default",
    "and",
    "or",
    "not",
    "xor",
    "eq",
    "ne",
    "lt",
    "gt",
    "le",
    "ge",
    "cmp",
    "undef",
    "defined",
    "bless",
    "ref",
    "print",
    "printf",
    "say",
    "map",
    "grep",
    "sort",
    "split",
    "join",
    "push",
    "pop",
    "shift",
    "unshift",
    "splice",
    "keys",
    "values",
    "each",
    "exists",
    "delete",
    "open",
    "close",
    "read",
    "write",
    "BEGIN",
    "END",
    "CHECK",
    "INIT",
    "UNITCHECK",
];

pub const KEYWORD_LENS: [u8; KEYWORD_COUNT] = compute_keyword_lens(&KEYWORD_NAMES);
pub const KW_MIN_LEN: usize = compute_kw_min_len(&KEYWORD_NAMES);
pub const KW_MAX_LEN: usize = compute_kw_max_len(&KEYWORD_NAMES);

const _: () = assert!(KEYWORD_NAMES.len() == KEYWORD_COUNT);
const _: () = assert!(KEYWORD_LENS.len() == KEYWORD_COUNT);

const TOK_IDENT: u8 = 3;
const TOK_HASH: u8 = 4;
const TOK_QUOTE: u8 = 5;
const TOK_APOSTROPHE: u8 = 6;
const TOK_BACKTICK: u8 = 7;


#[inline(always)]
const fn kw_hash(first: u8, second: u8, last: u8, len: usize) -> usize {
    let v = (first as usize)
        .wrapping_mul(873)
        .wrapping_add((second as usize).wrapping_mul(3146))
        .wrapping_add((last as usize).wrapping_mul(2425))
        .wrapping_add(len.wrapping_mul(3063));
    (v ^ (v >> 4)) & KW_SLOT_MASK
}

const KW_TABLE: [u8; 256] = {
    let mut table = [KW_EMPTY; 256];
    let mut i = 0;
    while i < KEYWORD_COUNT {
        let name = KEYWORD_NAMES[i].as_bytes();
        let len = KEYWORD_LENS[i] as usize;
        assert!(name.len() == len);
        assert!(len >= KW_MIN_LEN && len <= KW_MAX_LEN);
        let h = kw_hash(name[0], name[1], name[len - 1], len);
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
    table[b'#' as usize] = TOK_HASH;
    table[b'"' as usize] = TOK_QUOTE;
    table[b'\'' as usize] = TOK_APOSTROPHE;
    table[b'`' as usize] = TOK_BACKTICK;
    table
};

pub const TOKEN_LUT_LO: [i8; 32] = make_token_lut_lo(&TOKEN_KIND);

#[inline(always)]
fn skip_non_tokens_avx2(ptr: &[u8], i: usize) -> usize {
    skip_non_tokens_with_lut_avx2(ptr, i, &TOKEN_KIND, &TOKEN_LUT_LO)
}

#[cfg(test)]
#[inline(always)]
pub fn is_perl_extension(name: &[u8]) -> bool {
    super::detect_language(name, super::LANG_ALL_MASK) == Some(super::LangId::Perl)
}

pub fn parse_into(content: &[u8], acc: &mut ParseResult<KEYWORD_COUNT>) {
    let content_len = content.len();
    if content_len == 0 {
        return;
    }

    let mut line_state = 0u8;
    let mut i = 0usize;
    while i < content_len {
        if line_state == 0 {
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
            if content[i] == b'='
                && i + 1 < content_len
                && ((content[i + 1] >= b'a' && content[i + 1] <= b'z')
                    || (content[i + 1] >= b'A' && content[i + 1] <= b'Z'))
            {
                let mut pod_i = i;
                while pod_i < content_len {
                    let Some(pos) = find_byte_avx2(&content[pod_i..], b'\n') else {
                        line_state = STATE_COMMENT;
                        pod_i = content_len;
                        break;
                    };
                    let nl_abs = pod_i + pos;
                    acc.comment_lines += 1;
                    acc.lines_processed += 1;
                    pod_i = nl_abs + 1;
                    if pod_i + 3 < content_len && &content[pod_i..pod_i + 4] == b"=cut" {
                        if let Some(cut_pos) = find_byte_avx2(&content[pod_i..], b'\n') {
                            acc.comment_lines += 1;
                            acc.lines_processed += 1;
                            pod_i += cut_pos + 1;
                        } else {
                            line_state = STATE_COMMENT;
                            pod_i = content_len;
                        }
                        break;
                    }
                }
                i = pod_i;
                continue;
            }
        }

        if TOKEN_KIND[content[i] as usize] <= TOK_CODE && content[i] != b'\n' {
            line_state |= STATE_CODE;
            i += 1;
            if i < content_len && TOKEN_KIND[content[i] as usize] <= TOK_CODE && content[i] != b'\n'
            {
                i = skip_non_tokens_avx2(content, i);
            }
            if i >= content_len {
                break;
            }
        }

        match TOKEN_KIND[content[i] as usize] {
            TOK_NL => {
                if (line_state & STATE_CODE) != 0 {
                    acc.code_lines += 1;
                } else if (line_state & STATE_COMMENT) != 0 {
                    acc.comment_lines += 1;
                } else {
                    acc.blank_lines += 1;
                }
                acc.lines_processed += 1;
                line_state = 0;
                i += 1;
            }
            TOK_IDENT => {
                let start = i;
                i = skip_ident_cont_avx2(content, i + 1);
                debug_assert!(i > start && i <= content_len);
                let n = i - start;
                let cand = &content[start..i];

                if cand == b"__DATA__" || cand == b"__END__" {
                    if let Some(pos) = find_byte_avx2(&content[i..], b'\n') {
                        let nl_abs = i + pos;
                        acc.code_lines += 1;
                        acc.lines_processed += 1;
                        let rem_nls = count_newlines_avx2(&content[nl_abs + 1..]);
                        acc.comment_lines += rem_nls as i64;
                        acc.lines_processed += rem_nls as i64;
                        line_state = STATE_COMMENT;
                    }
                    break;
                }

                let is_sigil_or_method = if start > 0 {
                    let p1 = content[start - 1];
                    p1 == b'$'
                        || p1 == b'@'
                        || p1 == b'%'
                        || (start >= 2 && content[start - 2] == b'-' && p1 == b'>')
                } else {
                    false
                };

                line_state |= STATE_CODE;
                if !is_sigil_or_method && n.wrapping_sub(KW_MIN_LEN) <= KW_MAX_LEN - KW_MIN_LEN {
                    let h = kw_hash(cand[0], cand[1], cand[n - 1], n);
                    let id = KW_TABLE[h];
                    if id != KW_EMPTY {
                        let idx = id as usize;
                        debug_assert!(idx < KEYWORD_COUNT);
                        if KEYWORD_LENS[idx] as usize == n
                            && kw_match_2_to_16(cand, KEYWORD_NAMES[idx].as_bytes())
                        {
                            acc.keyword_counts[idx] += 1;
                            acc.total_keywords += 1;
                        }
                    }
                }
            }
            TOK_HASH => {
                line_state |= STATE_COMMENT;
                if let Some(pos) = find_byte_avx2(&content[i + 1..], b'\n') {
                    if (line_state & STATE_CODE) != 0 {
                        acc.code_lines += 1;
                    } else {
                        acc.comment_lines += 1;
                    }
                    acc.lines_processed += 1;
                    line_state = 0;
                    i += 1 + pos + 1;
                } else {
                    i = content_len;
                }
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
                let end_i = skip_literal(content, i + 1, b'`');
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
    finish_trailing_line(content, line_state, acc);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang::IDENT_CONT;

    pub fn find_kw_id(name: &str) -> usize {
        KEYWORD_NAMES.iter().position(|&kw| kw == name).unwrap()
    }

    #[test]
    fn test_perl_extensions() {
        for valid in [
            b"script.pl".as_slice(),
            b"Module.pm",
            b"test.t",
            b"UPPER.PL",
            b"UPPER.PM",
            b"UPPER.T",
            b"a/b/c/foo.bar.pl",
            b"a/b/c/Bar.pm",
        ] {
            assert!(is_perl_extension(valid));
        }

        for invalid in [
            b"main.c".as_slice(),
            b"main.cpp",
            b"main.rs",
            b"script.py",
            b"pl",
            b"pm",
            b".pl",
            b"",
        ] {
            assert!(!is_perl_extension(invalid));
        }
    }

    #[test]
    fn test_perl_keyword_table_and_mph() {
        assert_eq!(KEYWORD_COUNT, 73);
        let mut seen = [false; 256];
        for i in 0..KEYWORD_COUNT {
            let kw = KEYWORD_NAMES[i];
            let len = kw.len();
            assert_eq!(len, KEYWORD_LENS[i] as usize);
            assert!((2..=KW_MAX_LEN).contains(&len));
            let b = kw.as_bytes();
            let v = (b[0] as usize)
                .wrapping_mul(873)
                .wrapping_add((b[1] as usize).wrapping_mul(3146))
                .wrapping_add((b[len - 1] as usize).wrapping_mul(2425))
                .wrapping_add(len.wrapping_mul(3063));
            let h = (v ^ (v >> 4)) & KW_SLOT_MASK;
            assert!(!seen[h], "Collision at slot {} on {}", h, kw);
            seen[h] = true;
        }

        let non_keywords = [
            "ifs",
            "elsifs",
            "elses",
            "subroutine",
            "function",
            "packages",
            "returns",
            "while_loop",
            "for_each",
            "myself",
            "ourselves",
            "prints",
            "defines",
            "opens",
            "closes",
        ];
        for s in non_keywords {
            let mut res = ParseResult::default();
            parse_into(s.as_bytes(), &mut res);
            assert_eq!(res.total_keywords, 0, "False positive on {}", s);
        }

        for b in 0..=255u8 {
            let mut block = [b' '; 64];
            block[19] = b;
            let pos = skip_non_tokens_avx2(&block, 0);
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
    fn test_perl_comments_and_strings() {
        let src = concat!(
            "# perl single line comment with if return my sub\n",
            "my $x = 42;\n",
            "my $s1 = 'if return sub while';\n",
            "my $s2 = \"elsif unless foreach\";\n",
            "my $cmd = `exit die warn`;\n",
            "if ($x > 0) {\n",
            "    print \"$x\\n\"; # comment\n",
            "    return;\n",
            "}\n"
        );

        let mut res = ParseResult::default();
        parse_into(src.as_bytes(), &mut res);

        assert_eq!(res.keyword_counts[find_kw_id("my")], 4);
        assert_eq!(res.keyword_counts[find_kw_id("if")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("print")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("return")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("sub")], 0);
        assert_eq!(res.keyword_counts[find_kw_id("while")], 0);
        assert_eq!(res.keyword_counts[find_kw_id("elsif")], 0);
        assert_eq!(res.keyword_counts[find_kw_id("unless")], 0);
        assert_eq!(res.keyword_counts[find_kw_id("foreach")], 0);
        assert_eq!(res.keyword_counts[find_kw_id("exit")], 0);
        assert_eq!(res.keyword_counts[find_kw_id("die")], 0);
        assert_eq!(res.keyword_counts[find_kw_id("warn")], 0);
        assert_eq!(res.total_keywords, 7);
        assert_eq!(res.lines_processed, 9);
    }

    #[test]
    fn test_perl_pod_documentation() {
        let src = concat!(
            "package MyModule;\n",
            "use strict;\n",
            "use warnings;\n",
            "\n",
            "=head1 NAME\n",
            "\n",
            "MyModule - package with sub if return\n",
            "\n",
            "=cut\n",
            "\n",
            "sub hello {\n",
            "    return 1;\n",
            "}\n",
            "1;\n"
        );

        let mut res = ParseResult::default();
        parse_into(src.as_bytes(), &mut res);

        assert_eq!(res.keyword_counts[find_kw_id("package")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("use")], 2);
        assert_eq!(res.keyword_counts[find_kw_id("sub")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("return")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("if")], 0);
        assert_eq!(res.total_keywords, 5);
        assert_eq!(res.lines_processed, 14);
        assert_eq!(res.comment_lines, 5);
        assert_eq!(res.blank_lines, 2);
        assert_eq!(res.code_lines, 7);
    }

    #[test]
    fn test_perl_sigils_and_data_sections() {
        let src = concat!(
            "use strict;\n",
            "my $if = 10;\n",
            "my @return = (1, 2);\n",
            "my %while = ('a' => 1);\n",
            "$obj->sub();\n",
            "if ($if) { return $if; }\n",
            "__DATA__\n",
            "sub if while for return\n",
            "more raw data\n"
        );

        let mut res = ParseResult::default();
        parse_into(src.as_bytes(), &mut res);

        assert_eq!(res.keyword_counts[find_kw_id("use")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("my")], 3);
        assert_eq!(res.keyword_counts[find_kw_id("if")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("return")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("sub")], 0);
        assert_eq!(res.keyword_counts[find_kw_id("while")], 0);
        assert_eq!(res.keyword_counts[find_kw_id("for")], 0);
        assert_eq!(res.total_keywords, 6);
    }

    #[test]
    fn test_perl_lines_code_comments_blanks() {
        let src = concat!(
            "# perl script\n",
            "\n",
            "use strict;\n",
            "my $x = 10; # code with comment\n",
            "=pod\n",
            "documentation line 1\n",
            "documentation line 2\n",
            "=cut\n",
            "sub run {\n",
            "    return $x;\n",
            "}\n"
        );

        let mut res = ParseResult::default();
        parse_into(src.as_bytes(), &mut res);

        assert_eq!(res.lines_processed, 11);
        assert_eq!(res.blank_lines, 1);
        assert_eq!(res.comment_lines, 5);
        assert_eq!(res.code_lines, 5);
    }

    #[test]
    fn test_perl_avx2_boundaries_and_eof() {
        let len = 32 * 300 + 17;
        let mut buf = vec![b' '; len];
        let kw = b"foreach";
        let kw_len = kw.len();
        let mut placed = 0i64;
        let mut off = 23usize;
        while off + kw_len < len {
            buf[off..off + kw_len].copy_from_slice(kw);
            placed += 1;
            off += 64;
        }

        let mut res = ParseResult::default();
        parse_into(&buf, &mut res);
        assert_eq!(res.total_keywords, placed);
        assert_eq!(res.keyword_counts[find_kw_id("foreach")], placed);

        for start_off in 0..96 {
            for &kw_str in &["if", "my", "sub", "return", "foreach", "UNITCHECK"] {
                let mut b = vec![b' '; 160];
                let kw_bytes = kw_str.as_bytes();
                b[start_off..start_off + kw_bytes.len()].copy_from_slice(kw_bytes);
                let mut r = ParseResult::default();
                parse_into(&b, &mut r);
                assert_eq!(r.total_keywords, 1);
                assert_eq!(r.keyword_counts[find_kw_id(kw_str)], 1);
            }
        }

        for empty_or_tiny in [
            "",
            "#",
            "# no newline",
            "\"",
            "'",
            "`",
            "\"unclosed string",
            "'unclosed single",
            "`unclosed backtick",
            "$if",
            "@sub",
            "%while",
            "->return",
        ] {
            let mut res = ParseResult::default();
            parse_into(empty_or_tiny.as_bytes(), &mut res);
            assert_eq!(res.total_keywords, 0);
        }
    }
}
