use super::{
    compute_keyword_lens, compute_kw_max_len, compute_kw_min_len, finish_line_comment,
    handle_quoted_literal, make_token_kind, make_token_lut_lo, match_keyword_caseless_with,
    unified_parse_into, LangLexer, ParseResult, KW_EMPTY, STATE_CODE,
};

pub const KEYWORD_COUNT: usize = 92;
const KW_SLOT_MASK: usize = 511;

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
        let second = if len > 1 { name[1] } else { b' ' };
        let h = kw_hash(name[0], second, name[len / 2], name[len - 1], len);
        assert!(table[h] == KW_EMPTY);
        table[h] = i as u8;
        i += 1;
    }
    table
};

const TOK_IDENT: u8 = 3;
const TOK_EXCLAMATION: u8 = 4;
const TOK_QUOTE: u8 = 5;
const TOK_APOSTROPHE: u8 = 6;

pub const TOKEN_KIND: [u8; 256] = make_token_kind(
    TOK_IDENT,
    &[
        (b'!', TOK_EXCLAMATION),
        (b'"', TOK_QUOTE),
        (b'\'', TOK_APOSTROPHE),
    ],
);

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
    const HAS_LINE_START_HOOK: bool = true;

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
                let second = if c.len() > 1 {
                    c[1].to_ascii_lowercase()
                } else {
                    b' '
                };
                kw_hash(
                    c[0].to_ascii_lowercase(),
                    second,
                    c[c.len() / 2].to_ascii_lowercase(),
                    c[c.len() - 1].to_ascii_lowercase(),
                    c.len(),
                )
            },
        )
    }

    #[inline(always)]
    fn handle_line_start(
        content: &[u8],
        i: usize,
        line_state: &mut u8,
        acc: &mut ParseResult<KEYWORD_COUNT>,
    ) -> usize {
        if i > 0 && content[i - 1] != b'\n' {
            return i;
        }
        let b = content[i];
        if b == b'*' {
            return finish_line_comment(content, i + 1, line_state, acc);
        }
        if b == b'c' || b == b'C' {
            let content_len = content.len();
            if i + 1 < content_len {
                let next = content[i + 1];
                if next.is_ascii_alphanumeric()
                    || next == b'_'
                    || next == b'('
                    || next == b'%'
                    || next == b'['
                {
                    return i;
                }
                let mut k = i + 1;
                while k < content_len && (content[k] == b' ' || content[k] == b'\t') {
                    k += 1;
                }
                if k < content_len
                    && content[k] == b'='
                    && (k + 1 >= content_len || content[k + 1] != b'=')
                {
                    return i;
                }
            }
            return finish_line_comment(content, i + 1, line_state, acc);
        }
        i
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
            TOK_EXCLAMATION => finish_line_comment(content, i + 1, line_state, acc),
            TOK_QUOTE => handle_quoted_literal(content, i, b'"', line_state, acc),
            TOK_APOSTROPHE => handle_quoted_literal(content, i, b'\'', line_state, acc),
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
