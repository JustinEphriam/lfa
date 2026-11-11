use super::{
    compute_keyword_lens, compute_kw_max_len, compute_kw_min_len, count_newlines_avx2,
    find_byte_avx2, make_token_lut_lo, skip_literal, unified_parse_into, LangLexer, ParseResult,
    STATE_CODE, STATE_COMMENT, STATE_NONE, TOK_CODE, TOK_NL, TOK_WS,
};

pub const KEYWORD_COUNT: usize = 92;
const KW_SLOT_MASK: usize = 511;
const KW_EMPTY: u8 = 255;

pub const KEYWORD_NAMES: [&str; KEYWORD_COUNT] = [
    "allocate",
    "assign",
    "associate",
    "backspace",
    "block",
    "call",
    "case",
    "close",
    "common",
    "complex",
    "contains",
    "continue",
    "cycle",
    "data",
    "deallocate",
    "dimension",
    "do",
    "double",
    "elemental",
    "else",
    "elseif",
    "elsewhere",
    "end",
    "enddo",
    "endfunction",
    "endif",
    "endmodule",
    "endprogram",
    "endsubroutine",
    "entry",
    "equivalence",
    "exit",
    "external",
    "file",
    "flush",
    "forall",
    "format",
    "function",
    "generic",
    "goto",
    "if",
    "implicit",
    "import",
    "in",
    "include",
    "inout",
    "inquire",
    "integer",
    "intent",
    "interface",
    "intrinsic",
    "logical",
    "module",
    "namelist",
    "none",
    "nullify",
    "only",
    "open",
    "operator",
    "optional",
    "out",
    "parameter",
    "pointer",
    "precision",
    "print",
    "private",
    "procedure",
    "program",
    "public",
    "pure",
    "read",
    "real",
    "recursive",
    "result",
    "return",
    "rewind",
    "save",
    "select",
    "sequence",
    "stop",
    "submodule",
    "subroutine",
    "target",
    "then",
    "type",
    "use",
    "value",
    "volatile",
    "wait",
    "where",
    "while",
    "write",
];

pub const KEYWORD_LENS: [u8; KEYWORD_COUNT] = compute_keyword_lens(&KEYWORD_NAMES);
pub const KW_MIN_LEN: usize = compute_kw_min_len(&KEYWORD_NAMES);
pub const KW_MAX_LEN: usize = compute_kw_max_len(&KEYWORD_NAMES);

const _: () = assert!(KEYWORD_NAMES.len() == KEYWORD_COUNT);
const _: () = assert!(KEYWORD_LENS.len() == KEYWORD_COUNT);

#[inline(always)]
const fn kw_hash(first: u8, second: u8, mid: u8, last: u8, len: usize) -> usize {
    let val = (first as usize)
        .wrapping_mul(129)
        .wrapping_add((second as usize).wrapping_mul(25))
        .wrapping_add((mid as usize).wrapping_mul(105))
        .wrapping_add((last as usize).wrapping_mul(56))
        .wrapping_add(len.wrapping_mul(158));
    (val ^ (val >> 4)) & KW_SLOT_MASK
}

const KW_TABLE: [u8; 512] = {
    let mut table = [KW_EMPTY; 512];
    let mut i = 0;
    while i < KEYWORD_COUNT {
        let name = KEYWORD_NAMES[i].as_bytes();
        let len = KEYWORD_LENS[i] as usize;
        assert!(name.len() == len);
        assert!(len >= KW_MIN_LEN && len <= KW_MAX_LEN);
        let second = if len > 1 { name[1] } else { b' ' };
        let h = kw_hash(name[0], second, name[len / 2], name[len - 1], len);
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

const TOK_IDENT: u8 = 3;
const TOK_EXCLAMATION: u8 = 4;
const TOK_QUOTE: u8 = 5;
const TOK_APOSTROPHE: u8 = 6;

pub const TOKEN_KIND: [u8; 256] = {
    let mut t = [TOK_CODE; 256];
    t[b' ' as usize] = TOK_WS;
    t[b'\t' as usize] = TOK_WS;
    t[b'\r' as usize] = TOK_WS;
    t[0x0b_usize] = TOK_WS;
    t[0x0c_usize] = TOK_WS;
    t[b'\n' as usize] = TOK_NL;

    let mut c = b'a';
    while c <= b'z' {
        t[c as usize] = TOK_IDENT;
        c += 1;
    }
    let mut c = b'A';
    while c <= b'Z' {
        t[c as usize] = TOK_IDENT;
        c += 1;
    }
    t[b'_' as usize] = TOK_IDENT;

    t[b'!' as usize] = TOK_EXCLAMATION;
    t[b'"' as usize] = TOK_QUOTE;
    t[b'\'' as usize] = TOK_APOSTROPHE;
    t
};

pub const TOKEN_LUT_LO: [i8; 32] = make_token_lut_lo(&TOKEN_KIND);

#[cfg(test)]
#[inline(always)]
pub fn is_fortran_extension(name: &[u8]) -> bool {
    super::detect_language(name, super::LANG_ALL_MASK) == Some(super::LangId::Fortran)
}

pub struct FortranLexer;

impl LangLexer<KEYWORD_COUNT> for FortranLexer {
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
        let b = content[i];
        if b == b'c' || b == b'C' || b == b'*' {
            *line_state |= STATE_COMMENT;
            let Some(pos) = find_byte_avx2(&content[i + 1..], b'\n') else {
                return content.len();
            };
            let nl_idx = i + 1 + pos;
            acc.comment_lines += 1;
            acc.lines_processed += 1;
            *line_state = STATE_NONE;
            nl_idx + 1
        } else {
            i
        }
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
            let second = if cand_len > 1 {
                if cand[1].is_ascii_uppercase() {
                    cand[1] + 32
                } else {
                    cand[1]
                }
            } else {
                b' '
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

            let h = kw_hash(first, second, mid, last, cand_len);
            let idx = KW_TABLE[h] as usize;
            if idx < KEYWORD_COUNT
                && kw_match_case_insensitive(cand, KEYWORD_NAMES[idx].as_bytes())
            {
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
            TOK_EXCLAMATION => {
                *line_state |= STATE_COMMENT;
                let Some(pos) = find_byte_avx2(&content[i + 1..], b'\n') else {
                    return content_len;
                };
                let nl_idx = i + 1 + pos;
                if (*line_state & STATE_CODE) != 0 {
                    acc.code_lines += 1;
                } else if (*line_state & STATE_COMMENT) != 0 {
                    acc.comment_lines += 1;
                } else {
                    acc.blank_lines += 1;
                }
                acc.lines_processed += 1;
                *line_state = STATE_NONE;
                nl_idx + 1
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
            TOK_APOSTROPHE => {
                *line_state |= STATE_CODE;
                let end_i = skip_literal(content, i + 1, b'\'');
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
    unified_parse_into::<KEYWORD_COUNT, FortranLexer>(content, acc);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fortran_extensions() {
        assert!(is_fortran_extension(b"main.f90"));
        assert!(is_fortran_extension(b"main.f95"));
        assert!(is_fortran_extension(b"legacy.f"));
        assert!(is_fortran_extension(b"fixed.for"));
        assert!(!is_fortran_extension(b"main.rs"));
        assert!(!is_fortran_extension(b"main.c"));
    }

    #[test]
    fn test_fortran_keyword_table_and_mph() {
        for (i, name) in KEYWORD_NAMES.iter().enumerate() {
            let b = name.as_bytes();
            let second = if b.len() > 1 { b[1] } else { b' ' };
            let h = kw_hash(b[0], second, b[b.len() / 2], b[b.len() - 1], b.len());
            assert_eq!(KW_TABLE[h], i as u8);
        }
    }

    #[test]
    fn test_fortran_comments_and_strings() {
        let src = b"! modern line comment\nC legacy comment\nPROGRAM test\n  INTEGER :: i\n  PRINT *, 'Hello, World!'\nEND PROGRAM test\n";
        let mut acc = ParseResult::default();
        parse_into(src, &mut acc);
        assert_eq!(acc.comment_lines, 2);
        assert_eq!(acc.code_lines, 4);
        assert!(acc.total_keywords >= 4);
    }
}
