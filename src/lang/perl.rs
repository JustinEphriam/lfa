use super::{
    compute_keyword_lens, compute_kw_max_len, compute_kw_min_len, count_newlines_avx2,
    find_byte_avx2, finish_line_comment, handle_quoted_literal, kw_hash_fsl_add,
    make_kw_table_fsl_add, make_token_kind, make_token_lut_lo, match_keyword_with,
    unified_parse_into, LangLexer, ParseResult, STATE_CODE, STATE_COMMENT,
};

#[cfg(test)]
use super::TOK_CODE;

pub const KEYWORD_COUNT: usize = 73;
const KW_SLOT_MASK: usize = 255;

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

const TOK_IDENT: u8 = 3;
const TOK_HASH: u8 = 4;
const TOK_QUOTE: u8 = 5;
const TOK_APOSTROPHE: u8 = 6;
const TOK_BACKTICK: u8 = 7;

#[inline(always)]
const fn kw_hash(first: u8, second: u8, last: u8, len: usize) -> usize {
    kw_hash_fsl_add(
        first,
        second,
        last,
        len,
        [873, 3146, 2425, 3063],
        4,
        KW_SLOT_MASK,
    )
}

const KW_TABLE: [u8; 256] = make_kw_table_fsl_add(&KEYWORD_NAMES, [873, 3146, 2425, 3063], 4);

pub const TOKEN_KIND: [u8; 256] = make_token_kind(
    TOK_IDENT,
    &[
        (b'#', TOK_HASH),
        (b'"', TOK_QUOTE),
        (b'\'', TOK_APOSTROPHE),
        (b'`', TOK_BACKTICK),
    ],
);

pub const TOKEN_LUT_LO: [i8; 32] = make_token_lut_lo(&TOKEN_KIND);

#[cfg(test)]
#[inline(always)]
pub fn is_perl_extension(name: &[u8]) -> bool {
    super::detect_language(name, super::LANG_ALL_MASK) == Some(super::LangId::Perl)
}

pub struct PerlLexer;

impl LangLexer<KEYWORD_COUNT> for PerlLexer {
    const TOKEN_KIND: [u8; 256] = TOKEN_KIND;
    const TOKEN_LUT_LO: [i8; 32] = TOKEN_LUT_LO;
    const HAS_DOLLAR_IDENT: bool = false;
    const SUPPORTS_TEMPLATES: bool = false;
    const HAS_SPECIAL_IDENT: bool = true;
    const HAS_LINE_START_HOOK: bool = true;

    #[inline(always)]
    fn match_keyword(_cand: &[u8]) -> Option<usize> {
        None
    }

    #[inline(always)]
    fn handle_line_start(
        content: &[u8],
        i: usize,
        line_state: &mut u8,
        acc: &mut ParseResult<KEYWORD_COUNT>,
    ) -> usize {
        let content_len = content.len();
        if content[i] == b'='
            && i + 1 < content_len
            && ((content[i + 1] >= b'a' && content[i + 1] <= b'z')
                || (content[i + 1] >= b'A' && content[i + 1] <= b'Z'))
        {
            let mut pod_i = i;
            while pod_i < content_len {
                let Some(pos) = find_byte_avx2(&content[pod_i..], b'\n') else {
                    *line_state = STATE_COMMENT;
                    return content_len;
                };
                let nl_abs = pod_i + pos;
                acc.comment_lines += 1;
                acc.lines_processed += 1;
                pod_i = nl_abs + 1;
                if pod_i + 3 < content_len && &content[pod_i..pod_i + 4] == b"=cut" {
                    if let Some(cut_pos) = find_byte_avx2(&content[pod_i..], b'\n') {
                        acc.comment_lines += 1;
                        acc.lines_processed += 1;
                        return pod_i + cut_pos + 1;
                    } else {
                        *line_state = STATE_COMMENT;
                        return content_len;
                    }
                }
            }
            return pod_i;
        }
        i
    }

    #[inline(always)]
    fn handle_ident(
        content: &[u8],
        start: usize,
        end: usize,
        line_state: &mut u8,
        acc: &mut ParseResult<KEYWORD_COUNT>,
    ) -> usize {
        let content_len = content.len();
        let cand = &content[start..end];
        if cand == b"__DATA__" || cand == b"__END__" {
            if let Some(pos) = find_byte_avx2(&content[end..], b'\n') {
                let nl_abs = end + pos;
                acc.code_lines += 1;
                acc.lines_processed += 1;
                let rem_nls = count_newlines_avx2(&content[nl_abs + 1..]);
                acc.comment_lines += rem_nls as i64;
                acc.lines_processed += rem_nls as i64;
                *line_state = STATE_COMMENT;
            }
            return content_len;
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

        if is_sigil_or_method {
            return end;
        }

        if let Some(idx) = match_keyword_with(
            cand,
            KW_MIN_LEN,
            KW_MAX_LEN,
            &KW_TABLE,
            &KEYWORD_LENS,
            &KEYWORD_NAMES,
            |c| kw_hash(c[0], c[1], c[c.len() - 1], c.len()),
        ) {
            acc.keyword_counts[idx] += 1;
            acc.total_keywords += 1;
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
            TOK_HASH => finish_line_comment(content, i + 1, line_state, acc),
            TOK_QUOTE => handle_quoted_literal(content, i, b'"', line_state, acc),
            TOK_APOSTROPHE => handle_quoted_literal(content, i, b'\'', line_state, acc),
            TOK_BACKTICK => handle_quoted_literal(content, i, b'`', line_state, acc),
            _ => {
                *line_state |= STATE_CODE;
                i + 1
            }
        }
    }
}

pub fn parse_into(content: &[u8], acc: &mut ParseResult<KEYWORD_COUNT>) {
    unified_parse_into::<KEYWORD_COUNT, PerlLexer>(content, acc);
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
            let pos = super::super::skip_non_tokens_with_lut_avx2(&block, 0, &TOKEN_KIND, &TOKEN_LUT_LO);
            if TOKEN_KIND[b as usize] > TOK_CODE || b == b'\n' {
                assert_eq!(pos, 19);
            } else {
                assert_eq!(pos, block.len());
            }

            let mut block = [b'a'; 64];
            block[19] = b;
            let id_pos = super::super::skip_ident_cont_avx2(&block, 0);
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
