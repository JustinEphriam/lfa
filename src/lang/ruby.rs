use super::{
    compute_keyword_lens, compute_kw_max_len, compute_kw_min_len, count_newlines_avx2,
    find_byte_avx2, finish_line_comment, handle_quoted_literal, kw_hash_fml_add,
    make_kw_table_fml_add, make_token_kind, make_token_lut_lo, match_keyword_with,
    unified_parse_into, LangLexer, ParseResult, STATE_COMMENT,
};

pub const KEYWORD_COUNT: usize = 39;
const KW_SLOT_MASK: usize = 255;

pub const KEYWORD_NAMES: [&str; KEYWORD_COUNT] = [
    "alias",
    "and",
    "begin",
    "break",
    "case",
    "class",
    "def",
    "defined",
    "do",
    "else",
    "elsif",
    "end",
    "ensure",
    "false",
    "for",
    "if",
    "in",
    "module",
    "next",
    "nil",
    "not",
    "or",
    "redo",
    "rescue",
    "retry",
    "return",
    "self",
    "super",
    "then",
    "true",
    "undef",
    "unless",
    "until",
    "when",
    "while",
    "yield",
    "__ENCODING__",
    "__FILE__",
    "__LINE__",
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
const fn kw_hash(first: u8, mid: u8, last: u8, len: usize) -> usize {
    kw_hash_fml_add(first, mid, last, len, [1, 1, 9, 49], 1, KW_SLOT_MASK)
}

const KW_TABLE: [u8; 256] = make_kw_table_fml_add(&KEYWORD_NAMES, [1, 1, 9, 49], 1);

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
pub fn is_ruby_extension(name: &[u8]) -> bool {
    super::detect_language(name, super::LANG_ALL_MASK) == Some(super::LangId::Ruby)
}

pub struct RubyLexer;

impl LangLexer<KEYWORD_COUNT> for RubyLexer {
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
        if (i == 0 || content[i - 1] == b'\n')
            && i + 5 < content_len
            && &content[i..i + 6] == b"=begin"
        {
            let next_byte = if i + 6 < content_len {
                content[i + 6]
            } else {
                b'\n'
            };
            if next_byte == b' ' || next_byte == b'\t' || next_byte == b'\r' || next_byte == b'\n' {
                let mut block_i = i;
                while block_i < content_len {
                    let Some(pos) = find_byte_avx2(&content[block_i..], b'\n') else {
                        *line_state = STATE_COMMENT;
                        return content_len;
                    };
                    let nl_abs = block_i + pos;
                    acc.comment_lines += 1;
                    acc.lines_processed += 1;
                    block_i = nl_abs + 1;
                    if block_i + 3 < content_len && &content[block_i..block_i + 4] == b"=end" {
                        let term = if block_i + 4 < content_len {
                            content[block_i + 4]
                        } else {
                            b'\n'
                        };
                        if term == b' ' || term == b'\t' || term == b'\r' || term == b'\n' {
                            if let Some(end_pos) = find_byte_avx2(&content[block_i..], b'\n') {
                                acc.comment_lines += 1;
                                acc.lines_processed += 1;
                                return block_i + end_pos + 1;
                            } else {
                                *line_state = STATE_COMMENT;
                                return content_len;
                            }
                        }
                    }
                }
                return block_i;
            }
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
        if (start == 0 || content[start - 1] == b'\n') && cand == b"__END__" {
            let mut p = end;
            while p < content_len
                && (content[p] == b' ' || content[p] == b'\t' || content[p] == b'\r')
            {
                p += 1;
            }
            if p == content_len {
                return content_len;
            }
            if content[p] == b'\n' {
                acc.code_lines += 1;
                acc.lines_processed += 1;
                let rem_nls = count_newlines_avx2(&content[p + 1..]);
                acc.comment_lines += rem_nls as i64;
                acc.lines_processed += rem_nls as i64;
                *line_state = STATE_COMMENT;
                return content_len;
            }
        }

        let is_method_or_symbol = if start > 0 {
            let p1 = content[start - 1];
            p1 == b':' || (start >= 2 && content[start - 2] == b'.' && p1 != b'.') || p1 == b'.'
        } else {
            false
        };

        if is_method_or_symbol {
            return end;
        }

        let mut actual_end = end;
        if actual_end < content_len && content[actual_end] == b'?' {
            if cand == b"defined" {
                actual_end += 1;
            } else {
                return actual_end + 1;
            }
        } else if actual_end < content_len && content[actual_end] == b'!' {
            return actual_end + 1;
        }

        if let Some(idx) = match_keyword_with(
            cand,
            KW_MIN_LEN,
            KW_MAX_LEN,
            &KW_TABLE,
            &KEYWORD_LENS,
            &KEYWORD_NAMES,
            |c| kw_hash(c[0], c[c.len() / 2], c[c.len() - 1], c.len()),
        ) {
            acc.keyword_counts[idx] += 1;
            acc.total_keywords += 1;
        }

        actual_end
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
            _ => i + 1,
        }
    }
}

pub fn parse_into(content: &[u8], acc: &mut ParseResult<KEYWORD_COUNT>) {
    unified_parse_into::<KEYWORD_COUNT, RubyLexer>(content, acc);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ruby_extensions() {
        assert!(is_ruby_extension(b"test.rb"));
        assert!(is_ruby_extension(b"Gemfile.rb"));
        assert!(!is_ruby_extension(b"test.r"));
    }

    #[test]
    fn test_ruby_keyword_table_and_mph() {
        assert_eq!(KEYWORD_NAMES.len(), KEYWORD_COUNT);
        assert_eq!(KEYWORD_LENS.len(), KEYWORD_COUNT);
        for (i, name) in KEYWORD_NAMES.iter().enumerate() {
            let b = name.as_bytes();
            let h = kw_hash(b[0], b[b.len() / 2], b[b.len() - 1], b.len());
            assert_eq!(KW_TABLE[h], i as u8);
        }
    }

    #[test]
    fn test_ruby_comments_and_strings() {
        let code = b"# comment line 1\n# comment line 2\nclass Foo\n  def bar\n    puts \"hello #{name}\"\n  end\nend\n";
        let mut acc = ParseResult::<KEYWORD_COUNT>::default();
        parse_into(code, &mut acc);
        assert_eq!(acc.comment_lines, 2);
        assert_eq!(acc.code_lines, 5);
        assert_eq!(acc.lines_processed, 7);
        assert_eq!(acc.total_keywords, 4);
    }

    #[test]
    fn test_ruby_block_comments() {
        let code = b"=begin\nblock comment line 1\nblock comment line 2\n=end\nx = 1\n";
        let mut acc = ParseResult::<KEYWORD_COUNT>::default();
        parse_into(code, &mut acc);
        assert_eq!(acc.comment_lines, 4);
        assert_eq!(acc.code_lines, 1);
        assert_eq!(acc.lines_processed, 5);
    }

    #[test]
    fn test_ruby_data_end() {
        let code = b"puts 'hi'\n__END__\nsome trailing\ndata lines\n";
        let mut acc = ParseResult::<KEYWORD_COUNT>::default();
        parse_into(code, &mut acc);
        assert_eq!(acc.code_lines, 2);
        assert_eq!(acc.comment_lines, 2);
        assert_eq!(acc.lines_processed, 4);
    }
}
