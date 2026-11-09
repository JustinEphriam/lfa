use super::{count_newlines_avx2, find_byte_avx2, skip_literal, ParseResult};

pub const KEYWORD_COUNT: usize = 37;
const KW_SLOT_MASK: usize = 255;
const KW_MAX_LEN: usize = 8;
const KW_EMPTY: u8 = 255;

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

pub const KEYWORD_LENS: [u8; KEYWORD_COUNT] = [
    5, 4, 4, 3, 2, 6, 5, 5, 5, 4, 5, 8, 3, 3, 4, 4, 6, 7, 3, 4, 6, 2, 6, 2, 2, 6, 5, 8, 3, 2, 4, 5,
    6, 3, 5, 4, 5,
];

const _: () = assert!(KEYWORD_NAMES.len() == KEYWORD_COUNT);
const _: () = assert!(KEYWORD_LENS.len() == KEYWORD_COUNT);

const TOK_NONE: u8 = 0;
const TOK_IDENT: u8 = 1;
const TOK_HASH: u8 = 2;
const TOK_QUOTE: u8 = 3;
const TOK_APOSTROPHE: u8 = 4;

#[inline(always)]
const fn kw_hash(first: u8, last: u8, len: usize) -> usize {
    ((first as usize)
        .wrapping_add((last as usize).wrapping_mul(8))
        .wrapping_add(len.wrapping_mul(12)))
        & KW_SLOT_MASK
}

const KW_TABLE: [u8; 256] = {
    let mut table = [KW_EMPTY; 256];
    let mut i = 0;
    while i < KEYWORD_COUNT {
        let name = KEYWORD_NAMES[i].as_bytes();
        let len = KEYWORD_LENS[i] as usize;
        assert!(name.len() == len);
        assert!(len >= 2 && len <= KW_MAX_LEN);
        let h = kw_hash(name[0], name[len - 1], len);
        assert!(table[h] == KW_EMPTY);
        table[h] = i as u8;
        i += 1;
    }
    table
};

const TOKEN_KIND: [u8; 256] = {
    let mut table = [TOK_NONE; 256];
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
    table
};

const IDENT_CONT: [u8; 256] = {
    let mut table = [0u8; 256];
    let mut c = b'a';
    while c <= b'z' {
        table[c as usize] = 1;
        c += 1;
    }
    let mut c = b'A';
    while c <= b'Z' {
        table[c as usize] = 1;
        c += 1;
    }
    let mut c = b'0';
    while c <= b'9' {
        table[c as usize] = 1;
        c += 1;
    }
    table[b'_' as usize] = 1;
    table
};

#[inline(always)]
fn kw_match_len_2_to_8(cand: &[u8], kw: &[u8]) -> bool {
    let n = cand.len();
    if n != kw.len() || !(2..=KW_MAX_LEN).contains(&n) {
        return false;
    }
    if n == 8 {
        let a = u64::from_ne_bytes(cand[..8].try_into().unwrap());
        let b = u64::from_ne_bytes(kw[..8].try_into().unwrap());
        a == b
    } else if n >= 4 {
        let a1 = u32::from_ne_bytes(cand[..4].try_into().unwrap());
        let b1 = u32::from_ne_bytes(kw[..4].try_into().unwrap());
        let a2 = u32::from_ne_bytes(cand[n - 4..n].try_into().unwrap());
        let b2 = u32::from_ne_bytes(kw[n - 4..n].try_into().unwrap());
        ((a1 ^ b1) | (a2 ^ b2)) == 0
    } else {
        let a1 = u16::from_ne_bytes(cand[..2].try_into().unwrap());
        let b1 = u16::from_ne_bytes(kw[..2].try_into().unwrap());
        a1 == b1 && cand[n - 1] == kw[n - 1]
    }
}

#[inline(always)]
fn skip_non_tokens_avx2(ptr: &[u8], mut i: usize) -> usize {
    debug_assert!(i <= ptr.len());
    #[cfg(target_feature = "avx2")]
    {
        use safe_arch::*;
        let lut_lo = m256i::from([
            2_i8, 3, 11, 11, 3, 3, 3, 11, 3, 3, 3, 1, 1, 1, 1, 5, 2, 3, 11, 11, 3, 3, 3, 11, 3, 3,
            3, 1, 1, 1, 1, 5,
        ]);
        let lut_hi = m256i::from([
            0_i8, 0, 8, 0, 1, 6, 1, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 0, 1, 6, 1, 2, 0, 0, 0, 0,
            0, 0, 0, 0,
        ]);
        let mask_0f = set_splat_i8_m256i(0x0F);
        let zero = zeroed_m256i();

        let (chunks, _) = ptr[i..].as_chunks::<32>();
        for &chunk in chunks {
            let v = m256i::from(chunk);
            let hi = shr_imm_u16_m256i::<4>(v) & mask_0f;
            let m_lo = shuffle_av_i8z_half_m256i(lut_lo, v);
            let m_hi = shuffle_av_i8z_half_m256i(lut_hi, hi);
            let hit = m_lo & m_hi;
            let is_zero = cmp_eq_mask_i8_m256i(hit, zero);
            let mask = !(move_mask_i8_m256i(is_zero) as u32);
            if mask != 0 {
                return i + mask.trailing_zeros() as usize;
            }
            i += 32;
        }
    }
    while i < ptr.len() && TOKEN_KIND[ptr[i] as usize] == TOK_NONE {
        i += 1;
    }
    i
}

#[inline(always)]
fn skip_ident_cont_avx2(ptr: &[u8], mut i: usize) -> usize {
    debug_assert!(i <= ptr.len());
    #[cfg(target_feature = "avx2")]
    {
        use safe_arch::*;
        let lut_lo = m256i::from([
            10_i8, 11, 11, 11, 11, 11, 11, 11, 11, 11, 3, 1, 1, 1, 1, 5, 10, 11, 11, 11, 11, 11,
            11, 11, 11, 11, 3, 1, 1, 1, 1, 5,
        ]);
        let lut_hi = m256i::from([
            0_i8, 0, 0, 8, 1, 6, 1, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 1, 6, 1, 2, 0, 0, 0, 0,
            0, 0, 0, 0,
        ]);
        let mask_0f = set_splat_i8_m256i(0x0F);
        let zero = zeroed_m256i();

        let (chunks, _) = ptr[i..].as_chunks::<32>();
        for &chunk in chunks {
            let v = m256i::from(chunk);
            let hi = shr_imm_u16_m256i::<4>(v) & mask_0f;
            let m_lo = shuffle_av_i8z_half_m256i(lut_lo, v);
            let m_hi = shuffle_av_i8z_half_m256i(lut_hi, hi);
            let hit = m_lo & m_hi;
            let is_non_ident = cmp_eq_mask_i8_m256i(hit, zero);
            let mask = move_mask_i8_m256i(is_non_ident) as u32;
            if mask != 0 {
                return i + mask.trailing_zeros() as usize;
            }
            i += 32;
        }
    }
    while i < ptr.len() && IDENT_CONT[ptr[i] as usize] != 0 {
        i += 1;
    }
    i
}

#[inline(always)]
fn skip_triple_literal(content: &[u8], start_i: usize, quote: u8) -> usize {
    debug_assert!(start_i <= content.len());
    let content_len = content.len();
    let mut i = start_i;
    while i + 2 < content_len {
        let Some(pos) = find_byte_avx2(&content[i..content_len - 2], quote) else {
            return content_len;
        };
        let q_idx = i + pos;
        if content[q_idx + 1] == quote && content[q_idx + 2] == quote {
            let mut escapes = 0usize;
            let mut j = q_idx;
            while j > start_i && content[j - 1] == b'\\' {
                escapes += 1;
                j -= 1;
            }
            if (escapes & 1) == 0 {
                return q_idx + 3;
            }
        }
        i = q_idx + 1;
    }
    content_len
}

#[inline(always)]
pub fn is_python_extension(name: &[u8]) -> bool {
    let len = name.len();
    if len >= 3 && name[len - 3] == b'.' {
        let p = name[len - 2] | 0x20;
        let y = name[len - 1] | 0x20;
        return p == b'p' && y == b'y';
    }
    if len >= 4 && name[len - 4] == b'.' {
        let p = name[len - 3] | 0x20;
        let y = name[len - 2] | 0x20;
        let ext = name[len - 1] | 0x20;
        return p == b'p' && y == b'y' && (ext == b'i' || ext == b'w');
    }
    false
}

pub fn parse_into(content: &[u8], acc: &mut ParseResult<KEYWORD_COUNT>) {
    let content_len = content.len();
    if content_len == 0 {
        return;
    }

    acc.lines_processed += count_newlines_avx2(content) as i64;

    let mut i = 0usize;
    while i < content_len {
        if TOKEN_KIND[content[i] as usize] == TOK_NONE {
            i += 1;
            if i < content_len && TOKEN_KIND[content[i] as usize] == TOK_NONE {
                i = skip_non_tokens_avx2(content, i);
            }
            if i >= content_len {
                break;
            }
        }

        match TOKEN_KIND[content[i] as usize] {
            TOK_IDENT => {
                let start = i;
                i = skip_ident_cont_avx2(content, i + 1);
                debug_assert!(i > start && i <= content_len);
                let n = i - start;
                if n.wrapping_sub(2) <= KW_MAX_LEN - 2 {
                    let h = kw_hash(content[start], content[i - 1], n);
                    let id = KW_TABLE[h];
                    if id != KW_EMPTY {
                        let idx = id as usize;
                        debug_assert!(idx < KEYWORD_COUNT);
                        if KEYWORD_LENS[idx] as usize == n
                            && kw_match_len_2_to_8(
                                &content[start..i],
                                KEYWORD_NAMES[idx].as_bytes(),
                            )
                        {
                            acc.keyword_counts[idx] += 1;
                            acc.total_keywords += 1;
                        }
                    }
                }
            }
            TOK_HASH => {
                if let Some(pos) = find_byte_avx2(&content[i + 1..], b'\n') {
                    i += 1 + pos + 1;
                } else {
                    i = content_len;
                }
            }
            TOK_QUOTE => {
                if i + 2 < content_len && content[i + 1] == b'"' && content[i + 2] == b'"' {
                    i = skip_triple_literal(content, i + 3, b'"');
                } else {
                    i = skip_literal(content, i + 1, b'"');
                }
            }
            TOK_APOSTROPHE => {
                if i + 2 < content_len && content[i + 1] == b'\'' && content[i + 2] == b'\'' {
                    i = skip_triple_literal(content, i + 3, b'\'');
                } else {
                    i = skip_literal(content, i + 1, b'\'');
                }
            }
            _ => {
                i += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            assert!(kw_match_len_2_to_8(bytes, bytes));

            let mut res = ParseResult::default();
            parse_into(bytes, &mut res);
            assert_eq!(res.total_keywords, 1);
            assert_eq!(res.keyword_counts[i], 1);
            assert_eq!(res.lines_processed, 0);
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
            let pos = skip_non_tokens_avx2(&block, 0);
            if TOKEN_KIND[b as usize] != TOK_NONE {
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
        assert_eq!(res.lines_processed, 1);
    }
}
