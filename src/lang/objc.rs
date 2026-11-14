use super::{
    compute_keyword_lens, compute_kw_max_len, compute_kw_min_len, handle_c_slash_comments,
    handle_quoted_literal, kw_hash_fml_add, make_kw_table_fml_add, make_token_kind,
    make_token_lut_lo, match_keyword_caseless_with, unified_parse_into, LangLexer, ParseResult,
    STATE_CODE,
};

pub const KEYWORD_COUNT: usize = 43;
const KW_SLOT_MASK: usize = 127;

pub const KEYWORD_NAMES: [&str; KEYWORD_COUNT] = [
    "auto",
    "bool",
    "break",
    "case",
    "char",
    "class",
    "const",
    "continue",
    "default",
    "do",
    "double",
    "else",
    "enum",
    "extern",
    "float",
    "for",
    "goto",
    "id",
    "if",
    "imp",
    "instancetype",
    "int",
    "long",
    "nil",
    "no",
    "register",
    "return",
    "sel",
    "self",
    "short",
    "signed",
    "sizeof",
    "static",
    "struct",
    "super",
    "switch",
    "typedef",
    "union",
    "unsigned",
    "void",
    "volatile",
    "while",
    "yes",
];

pub const KEYWORD_LENS: [u8; KEYWORD_COUNT] = compute_keyword_lens(&KEYWORD_NAMES);
pub const KW_MIN_LEN: usize = compute_kw_min_len(&KEYWORD_NAMES);
pub const KW_MAX_LEN: usize = compute_kw_max_len(&KEYWORD_NAMES);

#[inline(always)]
const fn kw_hash(first: u8, mid: u8, last: u8, len: usize) -> usize {
    kw_hash_fml_add(first, mid, last, len, [7344, 3345, 5048, 8311], 5, KW_SLOT_MASK)
}

const KW_TABLE: [u8; 128] = make_kw_table_fml_add(&KEYWORD_NAMES, [7344, 3345, 5048, 8311], 5);

const TOK_IDENT: u8 = 3;
const TOK_SLASH: u8 = 4;
const TOK_QUOTE: u8 = 5;
const TOK_APOSTROPHE: u8 = 6;

pub const TOKEN_KIND: [u8; 256] = make_token_kind(
    TOK_IDENT,
    &[
        (b'@', TOK_IDENT),
        (b'/', TOK_SLASH),
        (b'"', TOK_QUOTE),
        (b'\'', TOK_APOSTROPHE),
    ],
);

pub const TOKEN_LUT_LO: [i8; 32] = make_token_lut_lo(&TOKEN_KIND);

pub struct ObjectiveCLexer;

impl LangLexer<KEYWORD_COUNT> for ObjectiveCLexer {
    const TOKEN_KIND: [u8; 256] = TOKEN_KIND;
    const TOKEN_LUT_LO: [i8; 32] = TOKEN_LUT_LO;
    const HAS_DOLLAR_IDENT: bool = false;
    const SUPPORTS_TEMPLATES: bool = false;

    #[inline(always)]
    fn match_keyword(cand: &[u8]) -> Option<usize> {
        let cand = if !cand.is_empty() && cand[0] == b'@' {
            &cand[1..]
        } else {
            cand
        };
        match_keyword_caseless_with(
            cand,
            KW_MIN_LEN,
            KW_MAX_LEN,
            &KW_TABLE,
            &KEYWORD_LENS,
            &KEYWORD_NAMES,
            |c| kw_hash(c[0].to_ascii_lowercase(), c[c.len() / 2].to_ascii_lowercase(), c[c.len() - 1].to_ascii_lowercase(), c.len()),
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
        match kind {
            TOK_SLASH => handle_c_slash_comments(content, i, line_state, acc),
            TOK_QUOTE | TOK_APOSTROPHE => {
                handle_quoted_literal(content, i, content[i], line_state, acc)
            }
            _ => {
                *line_state |= STATE_CODE;
                i + 1
            }
        }
    }
}

#[inline(always)]
fn contains_token_word(haystack: &[u8], token: &[u8]) -> bool {
    let mut i = 0;
    while i + token.len() <= haystack.len() {
        if &haystack[i..i + token.len()] == token {
            let next_ok = i + token.len() == haystack.len()
                || !(haystack[i + token.len()].is_ascii_alphanumeric()
                    || haystack[i + token.len()] == b'_');
            if next_ok {
                return true;
            }
        }
        i += 1;
    }
    false
}

#[inline(always)]
fn scan_ident(bytes: &[u8], start: usize) -> usize {
    let mut end = start;
    while end < bytes.len() && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_') {
        end += 1;
    }
    end
}

#[inline(always)]
fn is_c_brace_control_line(trimmed: &[u8]) -> bool {
    if trimmed.ends_with(b"{") {
        let mut k = trimmed.len() - 1;
        while k > 0 && (trimmed[k - 1] == b' ' || trimmed[k - 1] == b'\t') {
            k -= 1;
        }
        if k > 0 && trimmed[k - 1] == b')' {
            return true;
        }
        for kw in [b"else".as_slice(), b"do".as_slice()] {
            if trimmed.starts_with(kw) && k == kw.len() {
                return true;
            }
        }
    }
    if trimmed.starts_with(b"}") {
        let mut k = 1;
        while k < trimmed.len() && (trimmed[k] == b' ' || trimmed[k] == b'\t') {
            k += 1;
        }
        let after = &trimmed[k..];
        if after.starts_with(b"else")
            && (after.len() == 4 || matches!(after[4], b' ' | b'\t' | b'{'))
        {
            return true;
        }
    }
    false
}

pub fn is_objc_file(content: &[u8]) -> bool {
    let mut matlab_block_depth = 0usize;
    for raw_line in content.split(|&b| b == b'\n') {
        let mut line = raw_line;
        if line.ends_with(b"\r") {
            line = &line[..line.len() - 1];
        }
        let mut s = 0;
        while s < line.len() && (line[s] == b' ' || line[s] == b'\t') {
            s += 1;
        }
        let mut e = line.len();
        while e > s && (line[e - 1] == b' ' || line[e - 1] == b'\t') {
            e -= 1;
        }
        let trimmed = &line[s..e];
        if trimmed == b"%{" {
            matlab_block_depth += 1;
            continue;
        }
        if matlab_block_depth > 0 {
            if trimmed == b"%}" {
                matlab_block_depth -= 1;
            }
            continue;
        }
        if trimmed.is_empty() || trimmed[0] == b'%' {
            continue;
        }
        if is_c_brace_control_line(trimmed)
            || trimmed.starts_with(b"//")
            || trimmed.starts_with(b"/*")
            || trimmed.starts_with(b"- (")
            || trimmed.starts_with(b"+ (")
            || trimmed.starts_with(b"-(")
            || trimmed.starts_with(b"+(")
        {
            return true;
        }
        if trimmed[0] == b'#' {
            let mut p = 1;
            while p < trimmed.len() && (trimmed[p] == b' ' || trimmed[p] == b'\t') {
                p += 1;
            }
            let end = scan_ident(trimmed, p);
            if matches!(
                &trimmed[p..end],
                b"import"
                    | b"include"
                    | b"define"
                    | b"undef"
                    | b"ifdef"
                    | b"ifndef"
                    | b"if"
                    | b"elif"
                    | b"else"
                    | b"endif"
                    | b"pragma"
                    | b"error"
                    | b"warning"
                    | b"line"
            ) {
                return true;
            }
        }
        if trimmed[0] == b'@' {
            let end = scan_ident(trimmed, 1);
            if matches!(
                &trimmed[1..end],
                b"interface"
                    | b"implementation"
                    | b"protocol"
                    | b"end"
                    | b"class"
                    | b"property"
                    | b"synthesize"
                    | b"dynamic"
                    | b"autoreleasepool"
                    | b"synchronized"
                    | b"try"
                    | b"catch"
                    | b"finally"
                    | b"throw"
                    | b"selector"
                    | b"encode"
                    | b"compatibility_alias"
                    | b"defs"
                    | b"optional"
                    | b"required"
                    | b"public"
                    | b"package"
                    | b"protected"
                    | b"private"
                    | b"return"
                    | b"YES"
                    | b"NO"
                    | b"yes"
                    | b"no"
                    | b"true"
                    | b"false"
            ) {
                return true;
            }
        }
        if trimmed[0].is_ascii_alphabetic() || trimmed[0] == b'_' {
            let end = scan_ident(trimmed, 0);
            if end < trimmed.len()
                && matches!(trimmed[end], b' ' | b'\t' | b'*')
                && matches!(
                    &trimmed[..end],
                    b"void"
                        | b"int"
                        | b"char"
                        | b"short"
                        | b"long"
                        | b"float"
                        | b"double"
                        | b"typedef"
                        | b"static"
                        | b"extern"
                        | b"struct"
                        | b"union"
                        | b"enum"
                        | b"unsigned"
                        | b"signed"
                        | b"const"
                        | b"volatile"
                        | b"register"
                        | b"auto"
                        | b"goto"
                        | b"bool"
                        | b"BOOL"
                        | b"size_t"
                        | b"ssize_t"
                        | b"ptrdiff_t"
                        | b"intptr_t"
                        | b"uintptr_t"
                        | b"int8_t"
                        | b"int16_t"
                        | b"int32_t"
                        | b"int64_t"
                        | b"uint8_t"
                        | b"uint16_t"
                        | b"uint32_t"
                        | b"uint64_t"
                        | b"id"
                        | b"Class"
                        | b"SEL"
                        | b"IMP"
                        | b"instancetype"
                        | b"NSInteger"
                        | b"NSUInteger"
                        | b"CGFloat"
                )
            {
                let mut q = end;
                while q < trimmed.len() && (trimmed[q] == b' ' || trimmed[q] == b'\t') {
                    q += 1;
                }
                if q < trimmed.len()
                    && (trimmed[q].is_ascii_alphabetic()
                        || matches!(trimmed[q], b'_' | b'*' | b'(' | b'{'))
                {
                    return true;
                }
            }
        }
        if trimmed.windows(2).any(|w| w == b"@\"")
            || trimmed.windows(10).any(|w| w == b"@selector(" || w == b"@protocol(")
            || trimmed.windows(8).any(|w| w == b"@encode(")
            || contains_token_word(trimmed, b"@YES")
            || contains_token_word(trimmed, b"@NO")
        {
            return true;
        }
    }
    false
}

pub fn parse_into(content: &[u8], res: &mut ParseResult<KEYWORD_COUNT>) {
    unified_parse_into::<KEYWORD_COUNT, ObjectiveCLexer>(content, res);
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_objc_keyword_table_and_mph() {
        for (i, &name) in KEYWORD_NAMES.iter().enumerate() {
            let idx = ObjectiveCLexer::match_keyword(name.as_bytes());
            assert_eq!(idx, Some(i), "keyword {} failed match", name);
            let upper = name.to_ascii_uppercase();
            let idx_upper = ObjectiveCLexer::match_keyword(upper.as_bytes());
            assert_eq!(idx_upper, Some(i), "keyword {} upper failed match", upper);
        }
        assert_eq!(ObjectiveCLexer::match_keyword(b"notakeyword"), None);
        assert_eq!(ObjectiveCLexer::match_keyword(b"x"), None);
    }

    #[test]
    fn test_objc_comments_and_strings() {
        let objc = b"// Objective-C\n#import <Foundation/Foundation.h>\nint main() {\n    BOOL flag = YES;\n    /* block */\n    return 0;\n}\n";
        let mut res = ParseResult::default();
        parse_into(objc, &mut res);
        assert_eq!(res.lines_processed, 7);
        assert_eq!(res.code_lines, 5);
        assert_eq!(res.comment_lines, 2);
        assert_eq!(res.blank_lines, 0);
        assert!(res.total_keywords >= 2);
    }

    #[test]
    fn test_is_objc_file_disambiguation() {
        assert!(is_objc_file(b"@class Foo;\n@return @YES;\n"));
        assert!(is_objc_file(b"#import <Foundation/Foundation.h>\n"));
        assert!(is_objc_file(b"@interface Foo : NSObject\n@end\n"));
        assert!(is_objc_file(b"@implementation Foo\n- (void)bar {}\n@end\n"));
        assert!(is_objc_file(b"// C comment in ObjC\nx = @\"hello\";\n"));

        assert!(!is_objc_file(
            b"% MATLAB comment\nfunction y = f(x)\n  y = x + 1;\nend\n"
        ));
        assert!(!is_objc_file(b"f = @(x) x.^2;\nA = [1, 2; 3, 4];\n"));
        assert!(!is_objc_file(b"c = {1, 2};\nfunction y = f(x)\n  y = x;\nend\n"));
        assert!(!is_objc_file(b"c = {\n    1, 2;\n    3, 4\n};\n"));
        assert!(!is_objc_file(
            b"%{\n@interface Foo\nint main(void) {\n}\n%}\nfunction y = f(x)\n  y = double(x) + char(x);\nend\n"
        ));

        let mut padded = vec![b'\n'; 5100];
        padded.extend_from_slice(b"@implementation Foo\n@end\n");
        assert!(is_objc_file(&padded));

        assert!(is_objc_file(b"int main(void) {\n    return 0;\n}\n"));
        assert!(is_objc_file(b"static void helper(int x) {}\n"));
        assert!(is_objc_file(b"typedef struct { int a; } Foo;\n"));
        assert!(is_objc_file(b"#if DEBUG\n}\n#endif\n"));
        assert!(is_objc_file(
            b"char *msg = \"hi\";\ndouble x = 1.0;\nlong n = 2;\nfloat f = 3.0f;\nshort s = 4;\n"
        ));
        assert!(is_objc_file(
            b"if (x > 0) {\n    foo();\n} else {\n    bar();\n}\n"
        ));
    }
}
