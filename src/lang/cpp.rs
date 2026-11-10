use super::{
    count_newlines_avx2, find_byte_avx2, find_star_slash, kw_match_2_to_16, skip_ident_cont_avx2,
    skip_literal, ParseResult, IDENT_CONT,
};

pub const KEYWORD_COUNT: usize = 97;
const KW_SLOT_MASK: usize = 511;
const KW_MAX_LEN: usize = 16;
const KW_EMPTY: u8 = 255;

pub const KEYWORD_NAMES: [&str; KEYWORD_COUNT] = [
    "alignas",
    "alignof",
    "and",
    "and_eq",
    "asm",
    "atomic_cancel",
    "atomic_commit",
    "atomic_noexcept",
    "auto",
    "bitand",
    "bitor",
    "bool",
    "break",
    "case",
    "catch",
    "char",
    "char8_t",
    "char16_t",
    "char32_t",
    "class",
    "compl",
    "concept",
    "const",
    "consteval",
    "constexpr",
    "constinit",
    "const_cast",
    "continue",
    "co_await",
    "co_return",
    "co_yield",
    "decltype",
    "default",
    "delete",
    "do",
    "double",
    "dynamic_cast",
    "else",
    "enum",
    "explicit",
    "export",
    "extern",
    "false",
    "final",
    "float",
    "for",
    "friend",
    "goto",
    "if",
    "inline",
    "int",
    "long",
    "mutable",
    "namespace",
    "new",
    "noexcept",
    "not",
    "not_eq",
    "nullptr",
    "operator",
    "or",
    "or_eq",
    "override",
    "private",
    "protected",
    "public",
    "register",
    "reinterpret_cast",
    "requires",
    "return",
    "short",
    "signed",
    "sizeof",
    "static",
    "static_assert",
    "static_cast",
    "struct",
    "switch",
    "template",
    "this",
    "thread_local",
    "throw",
    "true",
    "try",
    "typedef",
    "typeid",
    "typename",
    "union",
    "unsigned",
    "using",
    "virtual",
    "void",
    "volatile",
    "wchar_t",
    "while",
    "xor",
    "xor_eq",
];

pub const KEYWORD_LENS: [u8; KEYWORD_COUNT] = {
    let mut lens = [0u8; KEYWORD_COUNT];
    let mut i = 0;
    while i < KEYWORD_COUNT {
        lens[i] = KEYWORD_NAMES[i].len() as u8;
        i += 1;
    }
    lens
};

const _: () = assert!(KEYWORD_NAMES.len() == KEYWORD_COUNT);
const _: () = assert!(KEYWORD_LENS.len() == KEYWORD_COUNT);

const TOK_WS: u8 = 0;
const TOK_NL: u8 = 1;
const TOK_CODE: u8 = 2;
const TOK_IDENT: u8 = 3;
const TOK_SLASH: u8 = 4;
const TOK_QUOTE: u8 = 5;
const TOK_APOSTROPHE: u8 = 6;

const STATE_CODE: u8 = 1;
const STATE_COMMENT: u8 = 2;

#[inline(always)]
const fn kw_hash(first: u8, mid: u8, last: u8, len: usize) -> usize {
    let v = (first as usize)
        .wrapping_mul(9)
        .wrapping_add((mid as usize).wrapping_mul(73))
        .wrapping_add((last as usize).wrapping_mul(403))
        .wrapping_add(len.wrapping_mul(151));
    (v ^ (v >> 4)) & KW_SLOT_MASK
}

const KW_TABLE: [u8; 512] = {
    let mut table = [KW_EMPTY; 512];
    let mut i = 0;
    while i < KEYWORD_COUNT {
        let name = KEYWORD_NAMES[i].as_bytes();
        let len = KEYWORD_LENS[i] as usize;
        assert!(name.len() == len);
        assert!(len >= 2 && len <= KW_MAX_LEN);
        let h = kw_hash(name[0], name[len >> 1], name[len - 1], len);
        assert!(table[h] == KW_EMPTY);
        table[h] = i as u8;
        i += 1;
    }
    table
};

const TOKEN_KIND: [u8; 256] = {
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
    table[b'/' as usize] = TOK_SLASH;
    table[b'"' as usize] = TOK_QUOTE;
    table[b'\'' as usize] = TOK_APOSTROPHE;
    table
};


#[inline(always)]
fn skip_non_tokens_avx2(ptr: &[u8], mut i: usize) -> usize {
    debug_assert!(i <= ptr.len());
    #[cfg(target_feature = "avx2")]
    {
        use safe_arch::*;
        let lut_lo = m256i::from([
            2_i8, 3, 11, 3, 3, 3, 3, 11, 3, 3, 19, 1, 1, 1, 1, 13, 2, 3, 11, 3, 3, 3, 3, 11, 3, 3,
            19, 1, 1, 1, 1, 13,
        ]);
        let lut_hi = m256i::from([
            16_i8, 0, 8, 0, 1, 6, 1, 2, 0, 0, 0, 0, 0, 0, 0, 0, 16, 0, 8, 0, 1, 6, 1, 2, 0, 0, 0,
            0, 0, 0, 0, 0,
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
    while i < ptr.len() && TOKEN_KIND[ptr[i] as usize] <= TOK_CODE && ptr[i] != b'\n' {
        i += 1;
    }
    i
}

#[inline(always)]
fn has_raw_string_prefix(content: &[u8], quote_idx: usize) -> bool {
    if quote_idx == 0 || content[quote_idx - 1] != b'R' {
        return false;
    }
    let r_idx = quote_idx - 1;
    if r_idx == 0 || IDENT_CONT[content[r_idx - 1] as usize] == 0 {
        return true;
    }
    let p1 = content[r_idx - 1];
    if (p1 == b'L' || p1 == b'u' || p1 == b'U')
        && (r_idx == 1 || IDENT_CONT[content[r_idx - 2] as usize] == 0)
    {
        return true;
    }
    if p1 == b'8'
        && r_idx >= 2
        && content[r_idx - 2] == b'u'
        && (r_idx == 2 || IDENT_CONT[content[r_idx - 3] as usize] == 0)
    {
        return true;
    }
    false
}

#[inline(always)]
fn try_skip_raw_string(content: &[u8], quote_idx: usize) -> Option<usize> {
    let content_len = content.len();
    let delim_start = quote_idx + 1;
    let max_scan = (delim_start + 17).min(content_len);
    let mut paren_idx = None;
    for (k, &b) in content[..max_scan].iter().enumerate().skip(delim_start) {
        if b == b'(' {
            paren_idx = Some(k);
            break;
        }
        if b == b')' || b == b'\\' || b == b'"' || b <= b' ' {
            return None;
        }
    }
    let open_paren = paren_idx?;
    let delim = &content[delim_start..open_paren];
    let delim_len = delim.len();
    let mut i = open_paren + 1;
    while i < content_len {
        let Some(pos) = find_byte_avx2(&content[i..], b')') else {
            return Some(content_len);
        };
        let close_paren = i + pos;
        let after_paren = close_paren + 1;
        if after_paren + delim_len < content_len
            && content[after_paren + delim_len] == b'"'
            && &content[after_paren..after_paren + delim_len] == delim
        {
            return Some(after_paren + delim_len + 1);
        }
        i = close_paren + 1;
    }
    Some(content_len)
}

#[cfg(test)]
#[inline(always)]
pub fn is_cpp_extension(name: &[u8]) -> bool {
    let len = name.len();
    if len < 3 {
        return false;
    }
    if len >= 4 && name[len - 4] == b'.' {
        let b0 = name[len - 3] | 0x20;
        let b1 = name[len - 2] | 0x20;
        let b2 = name[len - 1] | 0x20;
        return matches!(
            [b0, b1, b2],
            [b'c', b'p', b'p']
                | [b'c', b'x', b'x']
                | [b'c', b'+', b'+']
                | [b'h', b'p', b'p']
                | [b'h', b'x', b'x']
                | [b'h', b'+', b'+']
                | [b'i', b'p', b'p']
                | [b'i', b'n', b'l']
                | [b'i', b'x', b'x']
        );
    }
    if name[len - 3] == b'.' {
        let b0 = name[len - 2] | 0x20;
        let b1 = name[len - 1] | 0x20;
        return matches!([b0, b1], [b'c', b'c'] | [b'h', b'h']);
    }
    if len >= 5 && name[len - 5] == b'.' {
        let b0 = name[len - 4] | 0x20;
        let b1 = name[len - 3] | 0x20;
        let b2 = name[len - 2] | 0x20;
        let b3 = name[len - 1] | 0x20;
        return [b0, b1, b2, b3] == *b"cppm";
    }
    false
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
            while i < content_len && (content[i] == b' ' || content[i] == b'\t' || content[i] == b'\r') {
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
        }

        if TOKEN_KIND[content[i] as usize] <= TOK_CODE && content[i] != b'\n' {
            line_state |= STATE_CODE;
            i += 1;
            if i < content_len && TOKEN_KIND[content[i] as usize] <= TOK_CODE && content[i] != b'\n' {
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
                line_state |= STATE_CODE;
                let start = i;
                i = skip_ident_cont_avx2(content, i + 1);
                debug_assert!(i > start && i <= content_len);
                let n = i - start;
                if n.wrapping_sub(2) <= KW_MAX_LEN - 2 {
                    let h = kw_hash(content[start], content[start + (n >> 1)], content[i - 1], n);
                    let id = KW_TABLE[h];
                    if id != KW_EMPTY {
                        let idx = id as usize;
                        debug_assert!(idx < KEYWORD_COUNT);
                        if KEYWORD_LENS[idx] as usize == n
                            && kw_match_2_to_16(
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
            TOK_SLASH => {
                if i + 1 < content_len {
                    let next_b = content[i + 1];
                    if next_b == b'/' {
                        line_state |= STATE_COMMENT;
                        let mut j = i + 2;
                        while j < content_len {
                            let Some(pos) = find_byte_avx2(&content[j..], b'\n') else {
                                j = content_len;
                                break;
                            };
                            let nl_idx = j + pos;
                            let mut check = nl_idx;
                            if check > j && content[check - 1] == b'\r' {
                                check -= 1;
                            }
                            let is_spliced = check > j && content[check - 1] == b'\\';
                            if (line_state & STATE_CODE) != 0 {
                                acc.code_lines += 1;
                            } else if (line_state & STATE_COMMENT) != 0 {
                                acc.comment_lines += 1;
                            } else {
                                acc.blank_lines += 1;
                            }
                            acc.lines_processed += 1;
                            if is_spliced {
                                line_state = STATE_COMMENT;
                                j = nl_idx + 1;
                            } else {
                                line_state = 0;
                                j = nl_idx + 1;
                                break;
                            }
                        }
                        i = j;
                        continue;
                    } else if next_b == b'*' {
                        line_state |= STATE_COMMENT;
                        if let Some(pos) = find_star_slash(&content[i + 2..]) {
                            let star_slash_pos = i + 2 + pos;
                            if let Some(first_nl) = find_byte_avx2(&content[i + 2..star_slash_pos], b'\n') {
                                if (line_state & STATE_CODE) != 0 {
                                    acc.code_lines += 1;
                                } else {
                                    acc.comment_lines += 1;
                                }
                                acc.lines_processed += 1;
                                let first_nl_abs = i + 2 + first_nl;
                                let inter_nls = count_newlines_avx2(&content[first_nl_abs + 1..star_slash_pos]);
                                acc.comment_lines += inter_nls as i64;
                                acc.lines_processed += inter_nls as i64;
                                line_state = STATE_COMMENT;
                            }
                            i = star_slash_pos + 2;
                        } else {
                            if let Some(first_nl) = find_byte_avx2(&content[i + 2..], b'\n') {
                                if (line_state & STATE_CODE) != 0 {
                                    acc.code_lines += 1;
                                } else {
                                    acc.comment_lines += 1;
                                }
                                acc.lines_processed += 1;
                                let first_nl_abs = i + 2 + first_nl;
                                let rem_nls = count_newlines_avx2(&content[first_nl_abs + 1..]);
                                acc.comment_lines += rem_nls as i64;
                                acc.lines_processed += rem_nls as i64;
                            }
                            i = content_len;
                        }
                        continue;
                    }
                }
                line_state |= STATE_CODE;
                i += 1;
            }
            TOK_QUOTE => {
                line_state |= STATE_CODE;
                if has_raw_string_prefix(content, i)
                    && let Some(next_i) = try_skip_raw_string(content, i)
                {
                    let nl = count_newlines_avx2(&content[i + 1..next_i]);
                    if nl > 0 {
                        acc.code_lines += nl as i64;
                        acc.lines_processed += nl as i64;
                    }
                    i = next_i;
                    continue;
                }
                let end_i = skip_literal(content, i + 1, b'"');
                let nl = count_newlines_avx2(&content[i + 1..end_i]);
                if nl > 0 {
                    acc.code_lines += nl as i64;
                    acc.lines_processed += nl as i64;
                }
                i = end_i;
            }
            TOK_APOSTROPHE => {
                if i > 0 && i + 1 < content_len {
                    let prev = content[i - 1];
                    let next = content[i + 1];
                    if IDENT_CONT[prev as usize] != 0 && IDENT_CONT[next as usize] != 0 {
                        line_state |= STATE_CODE;
                        i += 1;
                        continue;
                    }
                }
                line_state |= STATE_CODE;
                let end_i = skip_literal(content, i + 1, b'\'');
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
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn find_kw_id(name: &str) -> usize {
        KEYWORD_NAMES.iter().position(|&kw| kw == name).unwrap()
    }

    #[test]
    fn test_cpp_keyword_table_and_mph() {
        assert_eq!(KEYWORD_COUNT, 97);
        let mut seen_slot = [false; 512];
        for i in 0..KEYWORD_COUNT {
            let kw = KEYWORD_NAMES[i];
            let len = kw.len();
            assert_eq!(len, KEYWORD_LENS[i] as usize);
            assert!((2..=KW_MAX_LEN).contains(&len));

            let bytes = kw.as_bytes();
            let h = kw_hash(bytes[0], bytes[len >> 1], bytes[len - 1], len);
            assert!(!seen_slot[h]);
            seen_slot[h] = true;
            assert_eq!(KW_TABLE[h], i as u8);
            assert!(kw_match_2_to_16(bytes, bytes));

            let mut res = ParseResult::default();
            parse_into(bytes, &mut res);
            assert_eq!(res.total_keywords, 1);
            assert_eq!(res.keyword_counts[i], 1);
            assert_eq!(res.lines_processed, 0);
        }

        let non_keywords = [
            "a",
            "i",
            "if_",
            "_if",
            "iff",
            "inta",
            "int_t",
            "classes",
            "namespaces",
            "templates",
            "typenames",
            "virtuals",
            "overrides",
            "finals",
            "reinterpret_casts",
            "static_casts",
            "dynamic_casts",
            "const_casts",
            "co_awaits",
            "co_returns",
            "co_yields",
            "char64_t",
            "_Bool",
            "_Generic",
            "_Static_assert",
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
    fn test_cpp_extensions() {
        for valid in [
            b"main.cpp".as_slice(),
            b"main.cc",
            b"main.cxx",
            b"main.c++",
            b"header.hpp",
            b"header.hh",
            b"header.hxx",
            b"header.h++",
            b"impl.ipp",
            b"math.inl",
            b"mod.ixx",
            b"mod.cppm",
            b"UPPER.CPP",
            b"UPPER.HPP",
            b"UPPER.CC",
            b"UPPER.HH",
            b"UPPER.CPPM",
        ] {
            assert!(is_cpp_extension(valid));
        }

        for invalid in [
            b"main.c".as_slice(),
            b"header.h",
            b"UPPER.C",
            b"UPPER.H",
            b"script.py",
            b"main.rs",
            b"cpp",
            b".c",
            b"",
        ] {
            assert!(!is_cpp_extension(invalid));
        }
    }

    #[test]
    fn test_cpp_features_and_raw_strings() {
        let src = concat!(
            "namespace demo {\n",
            "template <typename T> requires true\n",
            "class Widget final : public Base {\n",
            "public:\n",
            "    constexpr Widget() noexcept = default;\n",
            "    virtual void run() override {\n",
            "        auto s1 = R\"(class namespace template return if while \"quoted\")\";\n",
            "        auto s2 = R\"delim(reinterpret_cast<int>(x) ) \" )delim\";\n",
            "        if (true and not false) {\n",
            "            co_return;\n",
            "        }\n",
            "    }\n",
            "};\n",
            "}\n"
        );

        let mut res = ParseResult::default();
        parse_into(src.as_bytes(), &mut res);

        assert_eq!(res.keyword_counts[find_kw_id("namespace")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("template")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("typename")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("requires")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("true")], 2);
        assert_eq!(res.keyword_counts[find_kw_id("class")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("final")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("public")], 2);
        assert_eq!(res.keyword_counts[find_kw_id("constexpr")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("noexcept")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("default")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("virtual")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("void")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("override")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("auto")], 2);
        assert_eq!(res.keyword_counts[find_kw_id("if")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("and")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("not")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("false")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("co_return")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("reinterpret_cast")], 0);
        assert_eq!(res.keyword_counts[find_kw_id("return")], 0);
        assert_eq!(res.keyword_counts[find_kw_id("while")], 0);
        assert_eq!(res.total_keywords, 23);
        assert_eq!(res.lines_processed, 14);
    }

    #[test]
    fn test_cpp_raw_string_prefixes_and_delimiters() {
        let src = concat!(
            "auto a = u8R\"(if while return)\";\n",
            "auto b = LR\"(class struct union)\";\n",
            "auto c = uR\"(const_cast dynamic_cast)\";\n",
            "auto d = UR\"(co_await co_yield co_return)\";\n",
            "auto e = R\"1234567890123456( ) )123456789012345 \"if\" return )1234567890123456\";\n",
            "auto f = MY_MACRO_R\"(\\\")\"; if (true) return;\n",
            "auto g = R\"unclosed(class virtual override"
        );

        let mut res = ParseResult::default();
        parse_into(src.as_bytes(), &mut res);

        assert_eq!(res.keyword_counts[find_kw_id("auto")], 7);
        assert_eq!(res.keyword_counts[find_kw_id("if")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("true")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("return")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("class")], 0);
        assert_eq!(res.keyword_counts[find_kw_id("while")], 0);
        assert_eq!(res.total_keywords, 10);
    }

    #[test]
    fn test_cpp_avx2_boundaries_and_16_byte_keywords() {
        let len = 32 * 300 + 19;
        let mut buf = vec![b' '; len];
        let kw = b"reinterpret_cast";
        let kw_len = kw.len();
        let mut placed = 0i64;
        let mut off = 21usize;
        while off + kw_len < len {
            buf[off..off + kw_len].copy_from_slice(kw);
            placed += 1;
            off += 64;
        }

        let mut res = ParseResult::default();
        parse_into(&buf, &mut res);
        assert_eq!(res.total_keywords, placed);
        assert_eq!(res.keyword_counts[find_kw_id("reinterpret_cast")], placed);

        for start_off in 0..96 {
            for &kw_str in &[
                "or",
                "and",
                "auto",
                "class",
                "char16_t",
                "char32_t",
                "static_assert",
                "atomic_noexcept",
                "reinterpret_cast",
            ] {
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
    fn test_cpp_digit_separators_and_comments() {
        let src = concat!(
            "// spliced comment \\\n",
            "class Ignored {};\n",
            "constexpr int a = 0b1010'0101'1111;\n",
            "constexpr unsigned b = 0xDEAD'BEEF'CAFEu;\n",
            "/* block comment with namespace template typename */\n",
            "return a;\n"
        );

        let mut res = ParseResult::default();
        parse_into(src.as_bytes(), &mut res);

        assert_eq!(res.keyword_counts[find_kw_id("constexpr")], 2);
        assert_eq!(res.keyword_counts[find_kw_id("int")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("unsigned")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("return")], 1);
        assert_eq!(res.keyword_counts[find_kw_id("class")], 0);
        assert_eq!(res.total_keywords, 5);
        assert_eq!(res.lines_processed, 6);
    }

    #[test]
    fn test_cpp_lines_code_comments_blanks() {
        let src = concat!(
            "// header comment\n",
            "\n",
            "#include <iostream>\n",
            "const char *raw = R\"delim(\n",
            "line 1\n",
            "line 2 // not comment\n",
            ")delim\";\n",
            "/* multi\n",
            "comment */\n"
        );
        let mut res = ParseResult::default();
        parse_into(src.as_bytes(), &mut res);
        assert_eq!(res.lines_processed, 9);
        assert_eq!(res.blank_lines, 1);
        assert_eq!(res.comment_lines, 3);
        assert_eq!(res.code_lines, 5);
    }
}
