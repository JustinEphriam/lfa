use crate::doc::{
    detect_doc_format, parse_doc_into, DocFormatId, DocStats, ALL_DOC_FORMATS, DOC_FORMAT_COUNT,
};
use crate::lang::{
    detect_language, objc, parse_into, LangId, LangStats, ALL_LANGS, LANG_ALL_MASK, LANG_COUNT,
};

#[derive(Clone, Copy)]
pub struct RawQuoteSpec {
    pub open: &'static str,
    pub close: &'static str,
    pub single_line: bool,
    pub single_line_to_eol: bool,
    pub escapes_with_backslash: bool,
}

const fn rq(open: &'static str, close: &'static str) -> RawQuoteSpec {
    RawQuoteSpec {
        open,
        close,
        single_line: false,
        single_line_to_eol: false,
        escapes_with_backslash: false,
    }
}

const fn rqs(open: &'static str, close: &'static str) -> RawQuoteSpec {
    RawQuoteSpec {
        open,
        close,
        single_line: true,
        single_line_to_eol: false,
        escapes_with_backslash: false,
    }
}

const fn rqe(open: &'static str, close: &'static str) -> RawQuoteSpec {
    RawQuoteSpec {
        open,
        close,
        single_line: false,
        single_line_to_eol: false,
        escapes_with_backslash: true,
    }
}

const fn rql(open: &'static str) -> RawQuoteSpec {
    RawQuoteSpec {
        open,
        close: "",
        single_line: true,
        single_line_to_eol: true,
        escapes_with_backslash: false,
    }
}

#[derive(Clone, Copy)]
pub struct LangOracleSpec {
    pub lang: LangId,
    pub sample_filename: &'static str,
    pub caseless: bool,
    pub line_comments: &'static [&'static str],
    pub line_start_comments: &'static [&'static str],
    pub block_comments: &'static [(&'static str, &'static str, bool)],
    pub line_doc_blocks: &'static [(&'static str, &'static str)],
    pub quotes: &'static [(&'static str, bool)],
    pub raw_quotes: &'static [RawQuoteSpec],
    pub ident_prefixes: &'static [&'static str],
    pub suppress_sigils: &'static [&'static str],
    pub suppress_suffixes: &'static [&'static str],
    pub datum_comments: &'static [&'static str],
    pub eof_markers: &'static [&'static str],
    pub supports_spliced_line_comments: bool,
    pub supports_templates: bool,
    pub has_char_or_lifetime: bool,
    pub has_cpp_digit_sep: bool,
    pub has_matlab_transpose: bool,
    pub has_tex_escaped_percent: bool,
    pub has_perl_hash_sigil: bool,
    pub has_ruby_defined_q: bool,
}

#[derive(Clone, Copy)]
pub struct DocOracleSpec {
    pub doc: DocFormatId,
    pub sample_filename: &'static str,
    pub line_comments: &'static [&'static str],
    pub block_comments: &'static [(&'static str, &'static str)],
    pub has_quotes_masking_comments: bool,
    pub non_comment_directives: &'static [&'static str],
    pub neutral_line: &'static str,
}

macro_rules! define_lang_oracle_specs {
    (
        $(
            $variant:ident => {
                file: $file:expr,
                caseless: $caseless:expr,
                line: $line:expr,
                line_start: $line_start:expr,
                block: $block:expr,
                doc_block: $doc_block:expr,
                quotes: $quotes:expr,
                raw: $raw:expr,
                prefixes: $prefixes:expr,
                sigils: $sigils:expr,
                suffixes: $suffixes:expr,
                datum: $datum:expr,
                eof: $eof:expr,
                spliced: $spliced:expr,
                tmpl: $tmpl:expr,
                char_life: $char_life:expr,
                cpp_sep: $cpp_sep:expr,
                transpose: $transpose:expr,
                tex_pct: $tex_pct:expr,
                perl_hash: $perl_hash:expr,
                ruby_def: $ruby_def:expr $(,)?
            }
        ),* $(,)?
    ) => {
        pub const ORACLE_LANG_SPECS: &[LangOracleSpec] = &[
            $(
                LangOracleSpec {
                    lang: LangId::$variant,
                    sample_filename: $file,
                    caseless: $caseless,
                    line_comments: $line,
                    line_start_comments: $line_start,
                    block_comments: $block,
                    line_doc_blocks: $doc_block,
                    quotes: $quotes,
                    raw_quotes: $raw,
                    ident_prefixes: $prefixes,
                    suppress_sigils: $sigils,
                    suppress_suffixes: $suffixes,
                    datum_comments: $datum,
                    eof_markers: $eof,
                    supports_spliced_line_comments: $spliced,
                    supports_templates: $tmpl,
                    has_char_or_lifetime: $char_life,
                    has_cpp_digit_sep: $cpp_sep,
                    has_matlab_transpose: $transpose,
                    has_tex_escaped_percent: $tex_pct,
                    has_perl_hash_sigil: $perl_hash,
                    has_ruby_defined_q: $ruby_def,
                },
            )*
        ];
    };
}

macro_rules! define_doc_oracle_specs {
    (
        $(
            $variant:ident => {
                file: $file:expr,
                line: $line:expr,
                block: $block:expr,
                quoted: $quoted:expr,
                non_cmt: $non_cmt:expr,
                neutral: $neutral:expr $(,)?
            }
        ),* $(,)?
    ) => {
        pub const ORACLE_DOC_SPECS: &[DocOracleSpec] = &[
            $(
                DocOracleSpec {
                    doc: DocFormatId::$variant,
                    sample_filename: $file,
                    line_comments: $line,
                    block_comments: $block,
                    has_quotes_masking_comments: $quoted,
                    non_comment_directives: $non_cmt,
                    neutral_line: $neutral,
                },
            )*
        ];
    };
}

define_lang_oracle_specs! {
    C => {
        file: "sample.c", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: true, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Cpp => {
        file: "sample.cpp", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[rq("R\"(", ")\""), rq("R\"delim(", ")delim\"")],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: true, tmpl: false, char_life: false, cpp_sep: true, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Rust => {
        file: "sample.rs", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", true)], doc_block: &[],
        quotes: &[("\"", true)], raw: &[rq("r\"", "\""), rq("r#\"", "\"#"), rq("br#\"", "\"#")],
        prefixes: &[], sigils: &["r#"], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: true, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Python => {
        file: "sample.py", caseless: false,
        line: &["#"], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[rqe("\"\"\"", "\"\"\""), rqe("'''", "'''")],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Perl => {
        file: "sample.pl", caseless: false,
        line: &["#"], line_start: &[], block: &[], doc_block: &[("=pod", "=cut")],
        quotes: &[("\"", true), ("'", true), ("`", true)], raw: &[],
        prefixes: &[], sigils: &["$", "@", "%", "->"], suffixes: &[], datum: &[], eof: &["__END__", "__DATA__"],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: true, ruby_def: false,
    },
    Go => {
        file: "sample.go", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[rq("`", "`")],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Zig => {
        file: "sample.zig", caseless: false,
        line: &["//"], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[rql("\\\\")],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Odin => {
        file: "sample.odin", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", true)], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[rq("`", "`"), rqe("\"\"\"", "\"\"\"")],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    JavaScript => {
        file: "sample.js", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[rqe("`", "`")],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: true, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    TypeScript => {
        file: "sample.ts", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[rqe("`", "`")],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: true, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Lua => {
        file: "sample.lua", caseless: false,
        line: &["--"], line_start: &[], block: &[("--[[", "]]", false), ("--[=[", "]=]", false)], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[rq("[[", "]]"), rq("[=[", "]=]")],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Java => {
        file: "sample.java", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[rqe("\"\"\"", "\"\"\"")],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    CSharp => {
        file: "sample.cs", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[rqe("\"\"\"", "\"\"\""), rq("@\"", "\""), rqe("$\"", "\""), rq("$@\"", "\"")],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Ruby => {
        file: "sample.rb", caseless: false,
        line: &["#"], line_start: &[], block: &[], doc_block: &[("=begin", "=end")],
        quotes: &[("\"", true), ("'", true), ("`", true)], raw: &[],
        prefixes: &[], sigils: &[":", "."], suffixes: &["?", "!"], datum: &[], eof: &["__END__"],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: true,
    },
    Php => {
        file: "sample.php", caseless: true,
        line: &["//", "#"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true), ("'", true), ("`", true)], raw: &[],
        prefixes: &[], sigils: &["$", "->"], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Swift => {
        file: "sample.swift", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", true)], doc_block: &[],
        quotes: &[("\"", true)], raw: &[rqe("\"\"\"", "\"\"\"")],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Kotlin => {
        file: "sample.kt", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", true)], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[rqe("\"\"\"", "\"\"\"")],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Bash => {
        file: "sample.sh", caseless: false,
        line: &["#"], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("\"", true), ("'", true), ("`", true)], raw: &[],
        prefixes: &[], sigils: &["$"], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Sql => {
        file: "sample.sql", caseless: true,
        line: &["--", "#"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true), ("'", true), ("`", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Dart => {
        file: "sample.dart", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", true)], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[rqe("\"\"\"", "\"\"\""), rqe("'''", "'''"), rqs("r\"", "\""), rqs("r'", "'"), rqe("r\"\"\"", "\"\"\""), rqe("r'''", "'''")],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Scala => {
        file: "sample.scala", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", true)], doc_block: &[],
        quotes: &[("\"", true)], raw: &[rqe("\"\"\"", "\"\"\""), rqe("r\"", "\""), rqe("s\"", "\""), rqe("f\"", "\"")],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: true, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Haskell => {
        file: "sample.hs", caseless: false,
        line: &["--"], line_start: &[], block: &[("{-", "-}", true)], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: true, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    OCaml => {
        file: "sample.ml", caseless: false,
        line: &[], line_start: &[], block: &[("(*", "*)", true)], doc_block: &[],
        quotes: &[("\"", true)], raw: &[rq("{|", "|}"), rq("{raw|", "|raw}")],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: true, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Gleam => {
        file: "sample.gleam", caseless: false,
        line: &["//"], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    D => {
        file: "sample.d", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", true), ("/+", "+/", true)], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[rq("`", "`")],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Nim => {
        file: "sample.nim", caseless: false,
        line: &["#"], line_start: &[], block: &[("#[", "]#", true)], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[rqe("\"\"\"", "\"\"\"")],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Fortran => {
        file: "sample.f90", caseless: true,
        line: &["!"], line_start: &["* ", "c ", "C "], block: &[], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Elixir => {
        file: "sample.ex", caseless: false,
        line: &["#"], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[rqe("\"\"\"", "\"\"\""), rqe("'''", "'''")],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Erlang => {
        file: "sample.erl", caseless: false,
        line: &["%"], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Julia => {
        file: "sample.jl", caseless: false,
        line: &["#"], line_start: &[], block: &[("#=", "=#", true)], doc_block: &[],
        quotes: &[("\"", true)], raw: &[rqe("\"\"\"", "\"\"\"")],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: true, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    R => {
        file: "sample.r", caseless: false,
        line: &["#"], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[],
        prefixes: &[], sigils: &["."], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Matlab => {
        file: "sample.m", caseless: false,
        line: &["%"], line_start: &[], block: &[("%{", "%}", true)], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: true, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Solidity => {
        file: "sample.sol", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: true, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    FSharp => {
        file: "sample.fs", caseless: false,
        line: &["//"], line_start: &[], block: &[("(*", "*)", true)], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: true, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Mojo => {
        file: "sample.mojo", caseless: false,
        line: &["#"], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[rqe("\"\"\"", "\"\"\""), rqe("'''", "'''")],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Verilog => {
        file: "sample.v", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: true, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Cuda => {
        file: "sample.cu", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: true, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Wgsl => {
        file: "sample.wgsl", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: true, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Ada => {
        file: "sample.adb", caseless: true,
        line: &["--"], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Groovy => {
        file: "sample.groovy", caseless: false,
        line: &["//", "#"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[rqe("\"\"\"", "\"\"\""), rqe("'''", "'''")],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: true, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Clojure => {
        file: "sample.clj", caseless: false,
        line: &[";"], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &["-"], suffixes: &[], datum: &["#_"], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Assembly => {
        file: "sample.s", caseless: true,
        line: &[";", "#", "//"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[],
        prefixes: &["."], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Tcl => {
        file: "sample.tcl", caseless: false,
        line: &["#"], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[":"], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Scheme => {
        file: "sample.scm", caseless: false,
        line: &[";"], line_start: &[], block: &[("#|", "|#", true)], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &["-", "/"], suffixes: &[], datum: &["#;"], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Json => {
        file: "sample.json", caseless: false,
        line: &[], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Toml => {
        file: "sample.toml", caseless: false,
        line: &["#"], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Yaml => {
        file: "sample.yaml", caseless: true,
        line: &["#"], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Dockerfile => {
        file: "Dockerfile", caseless: true,
        line: &["#"], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Makefile => {
        file: "Makefile", caseless: false,
        line: &["#"], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[],
        prefixes: &[], sigils: &["-"], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    CMake => {
        file: "CMakeLists.txt", caseless: true,
        line: &["#"], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Glsl => {
        file: "sample.glsl", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Hlsl => {
        file: "sample.hlsl", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    ObjectiveC => {
        file: "sample.m", caseless: true,
        line: &["//"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[],
        prefixes: &["@"], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    V => {
        file: "sample.vsh", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    CommonLisp => {
        file: "sample.lisp", caseless: true,
        line: &[";"], line_start: &[], block: &[("#|", "|#", true)], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &["-", "*"], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Pascal => {
        file: "sample.pas", caseless: true,
        line: &["//"], line_start: &[], block: &[("{", "}", false), ("(*", "*)", false)], doc_block: &[],
        quotes: &[("'", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    PowerShell => {
        file: "sample.ps1", caseless: true,
        line: &["#"], line_start: &[], block: &[("<#", "#>", false)], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[],
        prefixes: &[], sigils: &["-"], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Fish => {
        file: "sample.fish", caseless: false,
        line: &["#"], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[],
        prefixes: &[], sigils: &["-"], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Awk => {
        file: "sample.awk", caseless: false,
        line: &["#"], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Hare => {
        file: "sample.ha", caseless: false,
        line: &["//"], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Html => {
        file: "sample.html", caseless: true,
        line: &[], line_start: &[], block: &[("<!--", "-->", false)], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[],
        prefixes: &[], sigils: &["-"], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Xml => {
        file: "sample.xml", caseless: true,
        line: &[], line_start: &[], block: &[("<!--", "-->", false)], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[],
        prefixes: &[], sigils: &["-", ":"], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Css => {
        file: "sample.css", caseless: true,
        line: &["//"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[],
        prefixes: &[], sigils: &["-"], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Graphql => {
        file: "sample.graphql", caseless: false,
        line: &["#"], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("\"", true)], raw: &[rqe("\"\"\"", "\"\"\"")],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Protobuf => {
        file: "sample.proto", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Cobol => {
        file: "sample.cob", caseless: true,
        line: &["*>"], line_start: &["000100* ", "      / "], block: &[], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[],
        prefixes: &[], sigils: &["-"], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Vhdl => {
        file: "sample.vhdl", caseless: true,
        line: &["--"], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    CHeader => {
        file: "sample.h", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: true, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    CppHeader => {
        file: "sample.hpp", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[rq("R\"(", ")\""), rq("R\"delim(", ")delim\"")],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: true, tmpl: false, char_life: false, cpp_sep: true, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Sass => {
        file: "sample.scss", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[],
        prefixes: &["@"], sigils: &["-"], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Nix => {
        file: "sample.nix", caseless: false,
        line: &["#"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true)], raw: &[rq("''", "''")],
        prefixes: &[], sigils: &["-"], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Hcl => {
        file: "sample.tf", caseless: false,
        line: &["#", "//"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &["-"], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Llvm => {
        file: "sample.ll", caseless: false,
        line: &[";"], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &["."], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Wat => {
        file: "sample.wat", caseless: false,
        line: &[";;"], line_start: &[], block: &[("(;", ";)", true)], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &["-", "."], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Tex => {
        file: "sample.tex", caseless: false,
        line: &["%"], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &["\\"], sigils: &["@"], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: true, perl_hash: false, ruby_def: false,
    },
    Elm => {
        file: "sample.elm", caseless: false,
        line: &["--"], line_start: &[], block: &[("{-", "-}", true)], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: true, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    PureScript => {
        file: "sample.purs", caseless: false,
        line: &["--"], line_start: &[], block: &[("{-", "-}", true)], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: true, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Racket => {
        file: "sample.rkt", caseless: false,
        line: &[";"], line_start: &[], block: &[("#|", "|#", true)], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &["-"], suffixes: &[], datum: &["#;"], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Sml => {
        file: "sample.sml", caseless: false,
        line: &[], line_start: &[], block: &[("(*", "*)", true)], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &["'"], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    SystemVerilog => {
        file: "sample.sv", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Meson => {
        file: "meson.build", caseless: false,
        line: &["#"], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[rqe("\"\"\"", "\"\"\""), rqe("'''", "'''")],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Starlark => {
        file: "BUILD.bazel", caseless: false,
        line: &["#"], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[rqe("\"\"\"", "\"\"\""), rqe("'''", "'''")],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Justfile => {
        file: "justfile", caseless: false,
        line: &["#"], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[rqe("\"\"\"", "\"\"\""), rqe("'''", "'''")],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Svelte => {
        file: "sample.svelte", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", false), ("<!--", "-->", false)], doc_block: &[],
        quotes: &[("\"", true), ("'", true), ("`", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Vue => {
        file: "sample.vue", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", false), ("<!--", "-->", false)], doc_block: &[],
        quotes: &[("\"", true), ("'", true), ("`", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Astro => {
        file: "sample.astro", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", false), ("<!--", "-->", false)], doc_block: &[],
        quotes: &[("\"", true), ("'", true), ("`", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    ObjectiveCpp => {
        file: "sample.mm", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[],
        prefixes: &["@"], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    ActionScript => {
        file: "sample.as", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Apex => {
        file: "sample.trigger", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    AutoHotkey => {
        file: "sample.ahk", caseless: true,
        line: &[";"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Crystal => {
        file: "sample.cr", caseless: false,
        line: &["#"], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Vala => {
        file: "sample.vala", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[rqe("\"\"\"", "\"\"\"")],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Chapel => {
        file: "sample.chpl", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[rqe("\"\"\"", "\"\"\""), rqe("'''", "'''")],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Pony => {
        file: "sample.pony", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", true)], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[rqe("\"\"\"", "\"\"\"")],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Ballerina => {
        file: "sample.bal", caseless: false,
        line: &["//", "#"], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    CoffeeScript => {
        file: "sample.coffee", caseless: false,
        line: &["#"], line_start: &[], block: &[("###", "###", false)], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[rqe("\"\"\"", "\"\"\""), rqe("'''", "'''")],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Less => {
        file: "sample.less", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[],
        prefixes: &["@"], sigils: &["-"], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    VimScript => {
        file: "sample.vim", caseless: false,
        line: &["\"", "#"], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("'", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Batch => {
        file: "sample.bat", caseless: true,
        line: &["::"], line_start: &["rem ", "REM "], block: &[], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &["@"], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Haxe => {
        file: "sample.hx", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Qml => {
        file: "sample.qml", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true), ("'", true), ("`", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Ron => {
        file: "sample.ron", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", true)], doc_block: &[],
        quotes: &[("\"", true)], raw: &[rq("r\"", "\""), rq("r#\"", "\"#"), rq("br#\"", "\"#")],
        prefixes: &[], sigils: &["r#"], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: true, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Thrift => {
        file: "sample.thrift", caseless: false,
        line: &["//", "#"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Prisma => {
        file: "sample.prisma", caseless: false,
        line: &["//"], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Kdl => {
        file: "sample.kdl", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", true)], doc_block: &[],
        quotes: &[("\"", true)], raw: &[rq("r\"", "\""), rq("r#\"", "\"#")],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    CapnProto => {
        file: "sample.capnp", caseless: false,
        line: &["#"], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Cue => {
        file: "sample.cue", caseless: false,
        line: &["//"], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Jsonnet => {
        file: "sample.jsonnet", caseless: false,
        line: &["//", "#"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Dhall => {
        file: "sample.dhall", caseless: false,
        line: &["--"], line_start: &[], block: &[("{-", "-}", true)], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Nickel => {
        file: "sample.ncl", caseless: false,
        line: &["#"], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Bicep => {
        file: "sample.bicep", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("'", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Wit => {
        file: "sample.wit", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", true)], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Move => {
        file: "sample.move", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Cairo => {
        file: "sample.cairo", caseless: false,
        line: &["//"], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Vyper => {
        file: "sample.vy", caseless: false,
        line: &["#"], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Fennel => {
        file: "sample.fnl", caseless: false,
        line: &[";"], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Hy => {
        file: "sample.hy", caseless: false,
        line: &[";"], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    EmacsLisp => {
        file: "sample.el", caseless: false,
        line: &[";"], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Raku => {
        file: "sample.raku", caseless: false,
        line: &["#"], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Forth => {
        file: "sample.fth", caseless: true,
        line: &["\\"], line_start: &[], block: &[("(", ")", false)], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Smalltalk => {
        file: "sample.st", caseless: false,
        line: &[], line_start: &[], block: &[("\"", "\"", false)], doc_block: &[],
        quotes: &[("'", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Wren => {
        file: "sample.wren", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", true)], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    FlatBuffers => {
        file: "sample.fbs", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Smithy => {
        file: "sample.smithy", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Janet => {
        file: "sample.janet", caseless: false,
        line: &["#"], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Idris => {
        file: "sample.idr", caseless: false,
        line: &["--"], line_start: &[], block: &[("{-", "-}", true)], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: true, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Modelica => {
        file: "sample.mo", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Stan => {
        file: "sample.stan", caseless: false,
        line: &["//", "#"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Shen => {
        file: "sample.shen", caseless: false,
        line: &[], line_start: &[], block: &[("\\*", "*\\", true)], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Firrtl => {
        file: "sample.fir", caseless: false,
        line: &[";"], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Abap => {
        file: "sample.abap", caseless: true,
        line: &["\""], line_start: &["*"], block: &[], doc_block: &[],
        quotes: &[("'", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Stata => {
        file: "sample.do", caseless: false,
        line: &["//"], line_start: &["*"], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    GDScript => {
        file: "sample.gd", caseless: false,
        line: &["#"], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[rqe("\"\"\"", "\"\"\""), rqe("'''", "'''")],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Prolog => {
        file: "sample.pro", caseless: false,
        line: &["%"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Agda => {
        file: "sample.agda", caseless: false,
        line: &["--"], line_start: &[], block: &[("{-", "-}", true)], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    OpenCL => {
        file: "sample.opencl", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: true, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    PostScript => {
        file: "sample.ps", caseless: false,
        line: &["%"], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("\"", true)], raw: &[rqe("(", ")")],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Grain => {
        file: "sample.gr", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Prql => {
        file: "sample.prql", caseless: false,
        line: &["#"], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[rqe("\"\"\"", "\"\"\""), rqe("'''", "'''")],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Q => {
        file: "sample.q", caseless: false,
        line: &["/"], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Slang => {
        file: "sample.slang", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: true, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    AutoIt => {
        file: "sample.au3", caseless: true,
        line: &[";"], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[],
        prefixes: &["$"], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Lean => {
        file: "sample.lean", caseless: false,
        line: &["--"], line_start: &[], block: &[("/-", "-/", true)], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Uiua => {
        file: "sample.ua", caseless: false,
        line: &["#"], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Bqn => {
        file: "sample.bqn", caseless: false,
        line: &["#"], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Cypher => {
        file: "sample.cyp", caseless: true,
        line: &["//"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Gherkin => {
        file: "sample.feature", caseless: true,
        line: &["#"], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("\"", true)], raw: &[rqe("\"\"\"", "\"\"\"")],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Eiffel => {
        file: "sample.e", caseless: true,
        line: &["--"], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    SuperCollider => {
        file: "sample.scd", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Red => {
        file: "sample.red", caseless: false,
        line: &[";"], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Koka => {
        file: "sample.koka", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Bluespec => {
        file: "sample.bsv", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Modula2 => {
        file: "sample.mod", caseless: true,
        line: &[], line_start: &[], block: &[("(*", "*)", true)], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Rexx => {
        file: "sample.rexx", caseless: true,
        line: &["--"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Rebol => {
        file: "sample.reb", caseless: false,
        line: &[";"], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Factor => {
        file: "sample.factor", caseless: false,
        line: &["!"], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Logtalk => {
        file: "sample.lgt", caseless: false,
        line: &["%"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Sparql => {
        file: "sample.sparql", caseless: true,
        line: &["#"], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Chuck => {
        file: "sample.ck", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Io => {
        file: "sample.io", caseless: false,
        line: &["#", "//"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Pike => {
        file: "sample.pike", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Dylan => {
        file: "sample.dylan", caseless: true,
        line: &["//"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    OpenSCAD => {
        file: "sample.scad", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    P4 => {
        file: "sample.p4", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Alloy => {
        file: "sample.als", caseless: false,
        line: &["//", "--"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    AppleScript => {
        file: "sample.applescript", caseless: true,
        line: &["--"], line_start: &[], block: &[("(*", "*)", true)], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Carp => {
        file: "sample.carp", caseless: false,
        line: &[";"], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    LiveScript => {
        file: "sample.ls", caseless: false,
        line: &["#"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    MoonScript => {
        file: "sample.moon", caseless: false,
        line: &["--"], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Modula3 => {
        file: "sample.m3", caseless: false,
        line: &[], line_start: &[], block: &[("(*", "*)", true)], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Oberon => {
        file: "sample.obn", caseless: false,
        line: &[], line_start: &[], block: &[("(*", "*)", true)], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Clean => {
        file: "sample.icl", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    HolyC => {
        file: "sample.hc", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Clarity => {
        file: "sample.clar", caseless: false,
        line: &[";"], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Pawn => {
        file: "sample.pwn", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    BlitzMax => {
        file: "sample.bmx", caseless: true,
        line: &["'"], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Asymptote => {
        file: "sample.asy", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Gnuplot => {
        file: "sample.gp", caseless: false,
        line: &["#"], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Processing => {
        file: "sample.pde", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Mlir => {
        file: "sample.mlir", caseless: false,
        line: &["//"], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Txl => {
        file: "sample.txl", caseless: false,
        line: &["%"], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Pact => {
        file: "sample.pact", caseless: false,
        line: &[";"], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Arduino => {
        file: "sample.ino", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Vba => {
        file: "sample.vba", caseless: true,
        line: &["'"], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("\"", false)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Ceylon => {
        file: "sample.ceylon", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Fantom => {
        file: "sample.fan", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Xtend => {
        file: "sample.xtend", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("\"", true), ("'", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Abnf => {
        file: "sample.abnf", caseless: true,
        line: &[";"], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("\"", false)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Al => {
        file: "sample.al", caseless: true,
        line: &["//"], line_start: &[], block: &[("/*", "*/", false)], doc_block: &[],
        quotes: &[("'", true), ("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    JLang => {
        file: "sample.ijs", caseless: false,
        line: &["NB."], line_start: &[], block: &[], doc_block: &[],
        quotes: &[("'", false)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Spice => {
        file: "sample.cir", caseless: true,
        line: &["$"], line_start: &["*"], block: &[], doc_block: &[],
        quotes: &[], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
    Ats => {
        file: "sample.dats", caseless: false,
        line: &["//"], line_start: &[], block: &[("/*", "*/", false), ("(*", "*)", true)], doc_block: &[],
        quotes: &[("\"", true)], raw: &[],
        prefixes: &[], sigils: &[], suffixes: &[], datum: &[], eof: &[],
        spliced: false, tmpl: false, char_life: false, cpp_sep: false, transpose: false, tex_pct: false, perl_hash: false, ruby_def: false,
    },
}

define_doc_oracle_specs! {
    Markdown => {
        file: "sample.md",
        line: &[],
        block: &[("<!--", "-->")],
        quoted: false,
        non_cmt: &["# Heading", "- item <!-- inline"],
        neutral: "Markdown content line 1",
    },
    PlainText => {
        file: "sample.txt",
        line: &[],
        block: &[],
        quoted: false,
        non_cmt: &["// not a comment", "# not a comment", "<!-- not a comment -->"],
        neutral: "Plain text content line 1",
    },
    Csv => {
        file: "sample.csv",
        line: &[],
        block: &[],
        quoted: false,
        non_cmt: &["a,b,c", "#header,val", "//col1,col2"],
        neutral: "1,2,3",
    },
    Svg => {
        file: "sample.svg",
        line: &[],
        block: &[("<!--", "-->")],
        quoted: false,
        non_cmt: &["<svg><!-- trailing", "<rect width=\"10\"/>"],
        neutral: "<g/>",
    },
    Ini => {
        file: "sample.ini",
        line: &["#", ";"],
        block: &[],
        quoted: false,
        non_cmt: &["[section]", "key = value # inline hash stays content"],
        neutral: "k = v",
    },
    RestructuredText => {
        file: "sample.rst",
        line: &[],
        block: &[],
        quoted: false,
        non_cmt: &[".. _target:", ".. [1] footnote", ".. |sub| replace:: x", ".. note:: body", ".. code-block:: rust"],
        neutral: "Paragraph text line",
    },
    Patch => {
        file: "sample.patch",
        line: &[],
        block: &[],
        quoted: false,
        non_cmt: &["--- a/x", "+++ b/x", "@@ -1 +1 @@"],
        neutral: "+added line",
    },
    Json5 => {
        file: "sample.json5",
        line: &["//"],
        block: &[("/*", "*/")],
        quoted: true,
        non_cmt: &["\"http://example.com /* not comment */\""],
        neutral: "key: 123,",
    },
    AsciiDoc => {
        file: "sample.adoc",
        line: &["//"],
        block: &[("////", "////")],
        quoted: false,
        non_cmt: &["= Document Title", "== Section Title", "* list item", "https://example.com"],
        neutral: "AsciiDoc paragraph text",
    },
    Org => {
        file: "sample.org",
        line: &["# "],
        block: &[("#+BEGIN_COMMENT", "#+END_COMMENT")],
        quoted: false,
        non_cmt: &["* Heading 1", "** Heading 2", "#+TITLE: Document", "#+AUTHOR: Someone"],
        neutral: "Org mode paragraph line",
    },
    BibTeX => {
        file: "sample.bib",
        line: &["%"],
        block: &[],
        quoted: false,
        non_cmt: &["@article{ref,", "title = {My Paper},", "year = {2024}"],
        neutral: "journal = {Nature},",
    },
    Properties => {
        file: "sample.properties",
        line: &["#", "!"],
        block: &[],
        quoted: false,
        non_cmt: &["key=value", "app.name = test", "database.url=localhost:5432"],
        neutral: "k = v",
    },
    Gettext => {
        file: "sample.po",
        line: &["#"],
        block: &[],
        quoted: false,
        non_cmt: &["msgid \"Hello\"", "msgstr \"Bonjour\"", "msgid_plural \"Hellos\""],
        neutral: "msgstr \"Monde\"",
    },
    Roff => {
        file: "sample.1",
        line: &[".\\\"", "'\\\""],
        block: &[],
        quoted: false,
        non_cmt: &[".TH GREP 1 \"2024-01-01\"", ".SH NAME", ".B grep", ".I pattern"],
        neutral: ".PP",
    },
    Typst => {
        file: "sample.typ",
        line: &["//"],
        block: &[("/*", "*/")],
        quoted: false,
        non_cmt: &["#set page(paper: \"a4\")", "= Introduction", "This is *Typst* text."],
        neutral: "Typst body paragraph",
    },
    Djot => {
        file: "sample.dj",
        line: &[],
        block: &[("{%", "%}")],
        quoted: false,
        non_cmt: &["# Heading 1", "_emphasis_", "`code block`"],
        neutral: "Djot paragraph line",
    },
    Textile => {
        file: "sample.textile",
        line: &["###."],
        block: &[],
        quoted: false,
        non_cmt: &["h1. Header 1", "p. A paragraph", "* item 1"],
        neutral: "Textile paragraph text",
    },
    Gemtext => {
        file: "sample.gmi",
        line: &[],
        block: &[],
        quoted: false,
        non_cmt: &["# Heading", "=> https://example.com Link", "* List item", "> Quote"],
        neutral: "Gemtext body line",
    },
    Pod => {
        file: "sample.pod",
        line: &[],
        block: &[("=begin comment", "=end comment")],
        quoted: false,
        non_cmt: &["=head1 NAME", "=head2 SYNOPSIS", "=item * bullet", "Some text in pod."],
        neutral: "Pod description paragraph",
    },
    MediaWiki => {
        file: "sample.wiki",
        line: &[],
        block: &[("<!--", "-->")],
        quoted: false,
        non_cmt: &["= Heading 1 =", "== Heading 2 ==", "[[Internal Link]]", "{{Template}}"],
        neutral: "MediaWiki paragraph text",
    },
    NdJson => {
        file: "sample.jsonl",
        line: &[],
        block: &[],
        quoted: false,
        non_cmt: &["{\"id\": 1, \"name\": \"test\"}", "{\"event\": \"login\"}"],
        neutral: "{\"key\": \"val\"}",
    },
    Avro => {
        file: "sample.avsc",
        line: &[],
        block: &[],
        quoted: false,
        non_cmt: &["{\"type\": \"record\", \"name\": \"User\"}", "{\"name\": \"age\", \"type\": \"int\"}"],
        neutral: "{\"type\": \"string\"}",
    },
    Nginx => {
        file: "nginx.conf",
        line: &["#"],
        block: &[],
        quoted: false,
        non_cmt: &["server {", "listen 80;", "server_name example.com;"],
        neutral: "location / { return 200; }",
    },
    Apache => {
        file: "httpd.conf",
        line: &["#"],
        block: &[],
        quoted: false,
        non_cmt: &["ServerRoot \"/etc/httpd\"", "<Directory \"/var/www\">", "Require all granted"],
        neutral: "RewriteEngine On",
    },
    SshConfig => {
        file: "ssh_config",
        line: &["#"],
        block: &[],
        quoted: false,
        non_cmt: &["Host *", "ServerAliveInterval 60", "IdentityFile ~/.ssh/id_ed25519"],
        neutral: "Port 22",
    },
    Crontab => {
        file: "crontab",
        line: &["#"],
        block: &[],
        quoted: false,
        non_cmt: &["0 0 * * * /usr/bin/backup.sh", "MAILTO=admin@example.com"],
        neutral: "*/5 * * * * /usr/bin/check.sh",
    },
    GitIgnore => {
        file: ".gitignore",
        line: &["#"],
        block: &[],
        quoted: false,
        non_cmt: &["target/", "*.log", "!important.log"],
        neutral: "/node_modules",
    },
    Dot => {
        file: "sample.dot",
        line: &["//", "#"],
        block: &[("/*", "*/")],
        quoted: false,
        non_cmt: &["digraph G {", "  a -> b;", "  b -> c [label=\"edge\"];", "}"],
        neutral: "node [shape=box];",
    },
    EditorConfig => {
        file: ".editorconfig",
        line: &["#", ";"],
        block: &[],
        quoted: false,
        non_cmt: &["root = true", "[*.rs]", "indent_style = space"],
        neutral: "indent_size = 4",
    },
    Http => {
        file: "sample.http",
        line: &["#", "//"],
        block: &[],
        quoted: false,
        non_cmt: &["GET https://example.com/api", "Content-Type: application/json"],
        neutral: "User-Agent: test",
    },
    WebVtt => {
        file: "sample.vtt",
        line: &["NOTE"],
        block: &[],
        quoted: false,
        non_cmt: &["WEBVTT", "00:00:01.000 --> 00:00:04.000", "Hello world"],
        neutral: "00:00:05.000 --> 00:00:08.000",
    },
    Srt => {
        file: "sample.srt",
        line: &[],
        block: &[],
        quoted: false,
        non_cmt: &["1", "00:00:01,000 --> 00:00:04,000", "Hello world"],
        neutral: "2",
    },
    Requirements => {
        file: "requirements.txt",
        line: &["#"],
        block: &[],
        quoted: false,
        non_cmt: &["requests>=2.28.0", "numpy==1.24.0", "pytest"],
        neutral: "flask",
    },
    PlantUML => {
        file: "sample.puml",
        line: &["'"],
        block: &[("/'", "'/")],
        quoted: false,
        non_cmt: &["@startuml", "Alice -> Bob : Authentication Request", "@enduml"],
        neutral: "Bob --> Alice : Response",
    },
    Mermaid => {
        file: "sample.mmd",
        line: &["%%"],
        block: &[],
        quoted: false,
        non_cmt: &["graph TD", "    A[Christmas] -->|Get money| B(Go shopping)", "    B --> C{Let me think}"],
        neutral: "    C -->|One| D[Laptop]",
    },
    Hosts => {
        file: "hosts",
        line: &["#"],
        block: &[],
        quoted: false,
        non_cmt: &["127.0.0.1 localhost", "::1 localhost ip6-localhost", "192.168.1.1 router"],
        neutral: "10.0.0.1 server",
    },
    Fstab => {
        file: "fstab",
        line: &["#"],
        block: &[],
        quoted: false,
        non_cmt: &["UUID=1234-5678 / ext4 defaults 0 1", "tmpfs /tmp tmpfs defaults 0 0"],
        neutral: "/dev/sda1 /boot ext2 defaults 0 2",
    },
    Resolv => {
        file: "resolv.conf",
        line: &["#", ";"],
        block: &[],
        quoted: false,
        non_cmt: &["nameserver 8.8.8.8", "nameserver 1.1.1.1", "search example.com", "options timeout:2"],
        neutral: "nameserver 9.9.9.9",
    },
    Log => {
        file: "sample.log",
        line: &[],
        block: &[],
        quoted: false,
        non_cmt: &["2024-01-01 12:00:00 [INFO] Server started", "2024-01-01 12:00:01 [ERROR] Connection lost"],
        neutral: "2024-01-01 12:00:02 [DEBUG] Retrying...",
    },
    Xsd => {
        file: "sample.xsd",
        line: &[],
        block: &[("<!--", "-->")],
        quoted: false,
        non_cmt: &["<xs:schema xmlns:xs=\"http://www.w3.org/2001/XMLSchema\">", "<xs:element name=\"note\"/>", "</xs:schema>"],
        neutral: "<xs:complexType/>",
    },
    Wsdl => {
        file: "sample.wsdl",
        line: &[],
        block: &[("<!--", "-->")],
        quoted: false,
        non_cmt: &["<definitions name=\"HelloService\">", "<message name=\"SayHello\">", "</definitions>"],
        neutral: "<portType name=\"Hello_PortType\"/>",
    },
    CMakeCache => {
        file: "CMakeCache.txt",
        line: &["#", "//"],
        block: &[],
        quoted: false,
        non_cmt: &["CMAKE_BUILD_TYPE:STRING=Release", "CMAKE_INSTALL_PREFIX:PATH=/usr/local"],
        neutral: "BUILD_SHARED_LIBS:BOOL=ON",
    },
    Cff => {
        file: "CITATION.cff",
        line: &["#"],
        block: &[],
        quoted: false,
        non_cmt: &["cff-version: 1.2.0", "title: My Software", "message: Please cite as below."],
        neutral: "version: 1.0.0",
    },
    GitConfig => {
        file: ".gitconfig",
        line: &["#", ";"],
        block: &[],
        quoted: false,
        non_cmt: &["[core]", "repositoryformatversion = 0", "filemode = true"],
        neutral: "[user]",
    },
    DockerCompose => {
        file: "docker-compose.yml",
        line: &["#"],
        block: &[],
        quoted: false,
        non_cmt: &["version: '3.8'", "services:", "  web:"],
        neutral: "    image: nginx",
    },
    Cabal => {
        file: "sample.cabal",
        line: &["--"],
        block: &[],
        quoted: false,
        non_cmt: &["name: mypackage", "version: 0.1.0.0", "build-type: Simple"],
        neutral: "cabal-version: >=1.10",
    },
    SubStationAlpha => {
        file: "sample.ssa",
        line: &[";"],
        block: &[],
        quoted: false,
        non_cmt: &["[Script Info]", "Title: Subtitle", "PlayResX: 640"],
        neutral: "[V4 Styles]",
    },
    Sami => {
        file: "sample.sami",
        line: &[],
        block: &[("<!--", "-->")],
        quoted: false,
        non_cmt: &["<SAMI>", "<HEAD>", "<TITLE>Example</TITLE>"],
        neutral: "<BODY>",
    },
    ICalendar => {
        file: "sample.ics",
        line: &[],
        block: &[],
        quoted: false,
        non_cmt: &["BEGIN:VCALENDAR", "VERSION:2.0", "PRODID:-//Example//EN"],
        neutral: "END:VCALENDAR",
    },
    TeXinfo => {
        file: "sample.texi",
        line: &["@c ", "@comment "],
        block: &[],
        quoted: false,
        non_cmt: &["\\input texinfo", "@setfilename sample.info", "@settitle Sample Manual"],
        neutral: "@bye",
    },
    GeoJson => {
        file: "sample.geojson",
        line: &[],
        block: &[],
        quoted: false,
        non_cmt: &["{\"type\": \"FeatureCollection\",", "\"features\": []", "}"],
        neutral: "{\"type\": \"Point\"}",
    },
    Kml => {
        file: "sample.kml",
        line: &[],
        block: &[("<!--", "-->")],
        quoted: false,
        non_cmt: &["<?xml version=\"1.0\" encoding=\"UTF-8\"?>", "<kml xmlns=\"http://www.opengis.net/kml/2.2\">", "<Placemark>"],
        neutral: "</kml>",
    },
    RssAtom => {
        file: "sample.rss",
        line: &[],
        block: &[("<!--", "-->")],
        quoted: false,
        non_cmt: &["<rss version=\"2.0\">", "<channel>", "<title>RSS Feed</title>"],
        neutral: "</channel>",
    },
    Xslt => {
        file: "sample.xslt",
        line: &[],
        block: &[("<!--", "-->")],
        quoted: false,
        non_cmt: &["<xsl:stylesheet>", "<xsl:template match=\"/\">"],
        neutral: "</xsl:stylesheet>",
    },
    Xcode => {
        file: "sample.pbxproj",
        line: &["//"],
        block: &[("/*", "*/")],
        quoted: false,
        non_cmt: &["archiveVersion = 1;", "classes = {", "objects = {"],
        neutral: "rootObject = 1234;",
    },
    Autoconf => {
        file: "configure.ac",
        line: &["#", "dnl "],
        block: &[],
        quoted: false,
        non_cmt: &["AC_INIT([pkg], [1.0])", "AC_CONFIG_SRCDIR([src/main.c])", "AM_INIT_AUTOMAKE"],
        neutral: "AC_OUTPUT",
    },
    Doxygen => {
        file: "Doxyfile",
        line: &["#"],
        block: &[],
        quoted: false,
        non_cmt: &["PROJECT_NAME = MyProject", "OUTPUT_DIRECTORY = doc", "GENERATE_HTML = YES"],
        neutral: "INPUT = src",
    },
    Sudoers => {
        file: "sudoers",
        line: &["#"],
        block: &[],
        quoted: false,
        non_cmt: &["root ALL=(ALL:ALL) ALL", "%sudo ALL=(ALL:ALL) ALL", "Defaults env_reset"],
        neutral: "Defaults secure_path=\"/usr/bin:/bin\"",
    },
    Passwd => {
        file: "passwd",
        line: &["#"],
        block: &[],
        quoted: false,
        non_cmt: &["root:x:0:0:root:/root:/bin/bash", "daemon:x:1:1:daemon:/usr/sbin:/usr/sbin/nologin"],
        neutral: "nobody:x:65534:65534:nobody:/nonexistent:/usr/sbin/nologin",
    },
    KnownHosts => {
        file: "known_hosts",
        line: &["#"],
        block: &[],
        quoted: false,
        non_cmt: &["github.com ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIOMqqnkVzrm0SdG6UOoqKLsabgH5C9okWi0dh2l9GKJl", "|1|abc= ecdsa-sha2-nistp256 AAAAE2VjZHNh..."],
        neutral: "gitlab.com ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQC...",
    },
    MesonWrap => {
        file: "sample.wrap",
        line: &["#"],
        block: &[],
        quoted: false,
        non_cmt: &["[wrap-file]", "directory = libfoo-1.0", "source_url = https://example.com/foo.tar.gz"],
        neutral: "source_filename = foo.tar.gz",
    },
    FontForge => {
        file: "sample.sfd",
        line: &["#"],
        block: &[],
        quoted: false,
        non_cmt: &["SplineFontDB: 3.0", "FontName: CustomFont", "FullName: Custom Font"],
        neutral: "EndSplineFont",
    },
    GitCommit => {
        file: "COMMIT_EDITMSG",
        line: &["#"],
        block: &[],
        quoted: false,
        non_cmt: &["feat: add new feature", "Initial commit description", "Resolves issue #123"],
        neutral: "Signed-off-by: Developer <dev@example.com>",
    },
}

const _: () = assert!(ORACLE_LANG_SPECS.len() == LANG_COUNT);
const _: () = assert!(ORACLE_DOC_SPECS.len() == DOC_FORMAT_COUNT);
const _: () = {
    let mut i = 0;
    while i < LANG_COUNT {
        assert!(ORACLE_LANG_SPECS[i].lang as usize == ALL_LANGS[i] as usize);
        i += 1;
    }
    let mut j = 0;
    while j < DOC_FORMAT_COUNT {
        assert!(ORACLE_DOC_SPECS[j].doc as usize == ALL_DOC_FORMATS[j] as usize);
        j += 1;
    }
};

struct ExpectedStats {
    lines: i64,
    code: i64,
    comments: i64,
    blanks: i64,
    total_keywords: i64,
    keyword_counts: Vec<i64>,
}

struct OracleBuilder {
    spec: &'static LangOracleSpec,
    kw_names: &'static [&'static str],
    buf: String,
    exp: ExpectedStats,
}

impl OracleBuilder {
    fn new(spec: &'static LangOracleSpec) -> Self {
        let kw_names = LangStats::keyword_names(spec.lang);
        Self {
            spec,
            kw_names,
            buf: String::new(),
            exp: ExpectedStats {
                lines: 0,
                code: 0,
                comments: 0,
                blanks: 0,
                total_keywords: 0,
                keyword_counts: vec![0i64; kw_names.len()],
            },
        }
    }

    fn count_keyword_token(&mut self, tok: &str) {
        if let Some(idx) = self.match_expected_kw(tok) {
            self.exp.keyword_counts[idx] += 1;
            self.exp.total_keywords += 1;
        }
    }

    fn match_expected_kw(&self, tok: &str) -> Option<usize> {
        if self.spec.caseless {
            self.kw_names
                .iter()
                .position(|&k| k.eq_ignore_ascii_case(tok))
        } else {
            self.kw_names.iter().position(|&k| k == tok)
        }
    }

    fn add_blanks(&mut self, lines: &[&str]) {
        for &l in lines {
            self.buf.push_str(l);
            self.buf.push('\n');
            self.exp.lines += 1;
            self.exp.blanks += 1;
        }
    }

    fn add_line_comment(&mut self, prefix: &str, payload: &str) {
        self.buf.push_str(prefix);
        self.buf.push(' ');
        self.buf.push_str(payload);
        self.buf.push('\n');
        self.exp.lines += 1;
        self.exp.comments += 1;
    }

    fn add_line_start_comment(&mut self, prefix: &str, payload: &str) {
        self.buf.push_str(prefix);
        self.buf.push_str(payload);
        self.buf.push('\n');
        self.exp.lines += 1;
        self.exp.comments += 1;
    }

    fn add_spliced_line_comment(&mut self, prefix: &str, hidden_kw: &str) {
        self.buf.push_str(prefix);
        self.buf.push_str(" spliced_1 \\\n");
        self.buf.push_str(hidden_kw);
        self.buf.push('\n');
        self.exp.lines += 2;
        self.exp.comments += 2;
    }

    fn add_single_block_comment(&mut self, open: &str, close: &str, payload: &str) {
        self.buf.push_str("  ");
        self.buf.push_str(open);
        self.buf.push(' ');
        self.buf.push_str(payload);
        self.buf.push(' ');
        self.buf.push_str(close);
        self.buf.push('\n');
        self.exp.lines += 1;
        self.exp.comments += 1;
    }

    fn add_multi_block_comment(&mut self, open: &str, close: &str, inner_lines: &[&str]) {
        self.buf.push_str(open);
        self.buf.push('\n');
        for &line in inner_lines {
            self.buf.push_str(line);
            self.buf.push('\n');
        }
        self.buf.push_str(close);
        self.buf.push('\n');
        let n = 2 + inner_lines.len() as i64;
        self.exp.lines += n;
        self.exp.comments += n;
    }

    fn add_nested_block_comment(&mut self, open: &str, close: &str, kw: &str) {
        self.buf.push_str(open);
        self.buf.push(' ');
        self.buf.push_str(kw);
        self.buf.push('\n');
        self.buf.push_str(open);
        self.buf.push(' ');
        self.buf.push_str(kw);
        self.buf.push(' ');
        self.buf.push_str(close);
        self.buf.push('\n');
        self.buf.push_str(kw);
        self.buf.push(' ');
        self.buf.push_str(close);
        self.buf.push('\n');
        self.exp.lines += 3;
        self.exp.comments += 3;
    }

    fn add_line_doc_block(&mut self, open: &str, close: &str, lines: &[&str]) {
        self.buf.push_str(open);
        self.buf.push('\n');
        self.exp.lines += 1;
        self.exp.comments += 1;
        for &l in lines {
            self.buf.push_str(l);
            self.buf.push('\n');
            self.exp.lines += 1;
            if l.bytes().all(|b| b == b' ' || b == b'\t' || b == b'\r') {
                self.exp.blanks += 1;
            } else {
                self.exp.comments += 1;
            }
        }
        self.buf.push_str(close);
        self.buf.push('\n');
        self.exp.lines += 1;
        self.exp.comments += 1;
    }

    fn add_code_line(&mut self, tokens: &[&str], pad: &str, trailing_comment: Option<&str>) {
        self.buf.push_str(pad);
        for (idx, &tok) in tokens.iter().enumerate() {
            if idx > 0 {
                self.buf.push(' ');
            }
            self.buf.push_str(tok);
            self.count_keyword_token(tok);
        }
        if let Some(cmt) = trailing_comment {
            self.buf.push(' ');
            self.buf.push_str(cmt);
        }
        self.buf.push('\n');
        self.exp.lines += 1;
        self.exp.code += 1;
    }

    fn add_mixed_block_and_code(&mut self, open: &str, close: &str, kw: &str) {
        self.buf.push_str("  ");
        self.buf.push_str(kw);
        self.buf.push(' ');
        self.buf.push_str(open);
        self.buf.push(' ');
        self.buf.push_str(kw);
        self.buf.push('\n');
        self.count_keyword_token(kw);

        self.buf.push_str("  ");
        self.buf.push_str(kw);
        self.buf.push('\n');

        self.buf.push_str("  ");
        self.buf.push_str(kw);
        self.buf.push(' ');
        self.buf.push_str(close);
        self.buf.push(' ');
        self.buf.push_str(kw);
        self.buf.push('\n');
        self.count_keyword_token(kw);

        self.exp.lines += 3;
        self.exp.code += 2;
        self.exp.comments += 1;
    }

    fn add_quoted_line(&mut self, q: &str, escape_bs: bool, kw: &str) {
        self.buf.push_str("  ");
        self.buf.push_str(q);
        self.buf.push_str("alpha ");
        if escape_bs {
            self.buf.push('\\');
            self.buf.push_str(q);
            self.buf.push(' ');
        }
        self.buf.push_str(kw);
        if escape_bs {
            self.buf.push_str(" \\\\");
        } else {
            self.buf.push_str(" beta");
        }
        self.buf.push_str(q);
        self.buf.push('\n');
        self.exp.lines += 1;
        self.exp.code += 1;
    }

    fn add_raw_quote(&mut self, rq_spec: RawQuoteSpec, kw: &str) {
        if rq_spec.single_line_to_eol {
            self.buf.push_str("  ");
            self.buf.push_str(rq_spec.open);
            self.buf.push(' ');
            self.buf.push_str(kw);
            self.buf.push('\n');
            self.exp.lines += 1;
            self.exp.code += 1;
            return;
        }
        if rq_spec.single_line {
            self.buf.push_str("  ");
            self.buf.push_str(rq_spec.open);
            self.buf.push_str(kw);
            self.buf.push_str(rq_spec.close);
            self.buf.push('\n');
            self.exp.lines += 1;
            self.exp.code += 1;
            return;
        }
        self.buf.push_str("  ");
        self.buf.push_str(rq_spec.open);
        self.buf.push_str(kw);
        if rq_spec.escapes_with_backslash {
            self.buf.push_str(" \\");
            self.buf.push_str(rq_spec.close);
        }
        self.buf.push('\n');
        self.buf.push('\n');
        self.buf.push_str(kw);
        if rq_spec.escapes_with_backslash {
            self.buf.push_str(" \\\\");
        }
        self.buf.push_str(rq_spec.close);
        self.buf.push('\n');
        self.exp.lines += 3;
        self.exp.code += 3;
    }

    fn add_datum_comment(&mut self, marker: &str, kw: &str) {
        self.buf.push_str(marker);
        self.buf.push_str(" (");
        self.buf.push_str(kw);
        self.buf.push('\n');
        self.buf.push_str("  ");
        self.buf.push_str(kw);
        self.buf.push_str(")\n");
        self.exp.lines += 2;
        self.exp.comments += 2;
    }

    fn add_sigil_suppressed(&mut self, sigil: &str, kw: &str) {
        self.buf.push_str("  ");
        self.buf.push_str(sigil);
        self.buf.push_str(kw);
        self.buf.push('\n');
        self.exp.lines += 1;
        self.exp.code += 1;
    }

    fn add_prefixed_keyword(&mut self, prefix: &str, kw: &str) {
        self.buf.push_str("  ");
        self.buf.push_str(prefix);
        self.buf.push_str(kw);
        self.buf.push('\n');
        self.count_keyword_token(kw);
        self.exp.lines += 1;
        self.exp.code += 1;
    }

    fn add_suffixed_suppressed(&mut self, suffix: &str, kw: &str) {
        self.buf.push_str("  ");
        self.buf.push_str(kw);
        self.buf.push_str(suffix);
        self.buf.push('\n');
        self.exp.lines += 1;
        self.exp.code += 1;
    }

    fn add_template_interp(&mut self, kw: &str) {
        self.buf.push_str("  `tmpl ");
        self.buf.push_str(kw);
        self.buf.push_str(" ${ ");
        self.buf.push_str(kw);
        self.count_keyword_token(kw);
        self.buf.push_str(" + `{}` } ");
        self.buf.push_str(kw);
        self.buf.push_str("`\n");
        self.exp.lines += 1;
        self.exp.code += 1;
    }

    fn add_eof_marker_tail(&mut self, marker: &str, kw: &str) {
        self.buf.push_str(marker);
        self.buf.push('\n');
        self.exp.lines += 1;
        self.exp.code += 1;

        self.buf.push_str(kw);
        self.buf.push('\n');
        self.exp.lines += 1;
        self.exp.comments += 1;

        self.buf.push_str("   \n");
        self.exp.lines += 1;
        self.exp.blanks += 1;

        self.buf.push_str(kw);
        self.buf.push('\n');
        self.exp.lines += 1;
        self.exp.comments += 1;
    }

    fn verify(&self, label: &str) {
        let mut stats = LangStats::default();
        parse_into(self.spec.lang, self.buf.as_bytes(), &mut stats);
        let lang = self.spec.lang;
        assert_eq!(
            stats.lines(lang),
            self.exp.lines,
            "{} ({}) lines mismatch",
            lang.name(),
            label
        );
        assert_eq!(
            stats.code_lines(lang),
            self.exp.code,
            "{} ({}) code_lines mismatch",
            lang.name(),
            label
        );
        assert_eq!(
            stats.comment_lines(lang),
            self.exp.comments,
            "{} ({}) comment_lines mismatch",
            lang.name(),
            label
        );
        assert_eq!(
            stats.blank_lines(lang),
            self.exp.blanks,
            "{} ({}) blank_lines mismatch",
            lang.name(),
            label
        );
        assert_eq!(
            stats.total_keywords(lang),
            self.exp.total_keywords,
            "{} ({}) total_keywords mismatch",
            lang.name(),
            label
        );
        assert_eq!(
            stats.keyword_counts(lang),
            self.exp.keyword_counts.as_slice(),
            "{} ({}) per-keyword counts mismatch",
            lang.name(),
            label
        );
    }
}

fn verify_line_invariants(lang: LangId, content: &[u8]) {
    let mut stats = LangStats::default();
    parse_into(lang, content, &mut stats);
    let lines = stats.lines(lang);
    let code = stats.code_lines(lang);
    let comments = stats.comment_lines(lang);
    let blanks = stats.blank_lines(lang);
    assert_eq!(
        lines,
        code + comments + blanks,
        "{} line sum invariant failed",
        lang.name()
    );
    if !content.is_empty() && content.last() != Some(&b'\n') {
        let mut with_nl = content.to_vec();
        with_nl.push(b'\n');
        let mut stats_nl = LangStats::default();
        parse_into(lang, &with_nl, &mut stats_nl);
        assert_eq!(
            stats_nl.lines(lang),
            lines,
            "{} +nl lines mismatch",
            lang.name()
        );
        assert_eq!(
            stats_nl.code_lines(lang),
            code,
            "{} +nl code mismatch",
            lang.name()
        );
        assert_eq!(
            stats_nl.comment_lines(lang),
            comments,
            "{} +nl comments mismatch",
            lang.name()
        );
        assert_eq!(
            stats_nl.blank_lines(lang),
            blanks,
            "{} +nl blanks mismatch",
            lang.name()
        );
        assert_eq!(
            stats_nl.total_keywords(lang),
            stats.total_keywords(lang),
            "{} +nl total_keywords mismatch",
            lang.name()
        );
        assert_eq!(
            stats_nl.keyword_counts(lang),
            stats.keyword_counts(lang),
            "{} +nl keyword_counts mismatch",
            lang.name()
        );
    }
}

fn build_torture_program(spec: &'static LangOracleSpec) -> OracleBuilder {
    let mut b = OracleBuilder::new(spec);
    let kw_names = LangStats::keyword_names(spec.lang);
    let first_kw = kw_names[0];
    let second_kw = kw_names[1.min(kw_names.len() - 1)];

    b.add_blanks(&["", "   ", "\t  \r"]);

    for (idx, &kw) in kw_names.iter().enumerate() {
        let pad = if idx % 3 == 0 {
            "  "
        } else if idx % 3 == 1 {
            "                                "
        } else {
            "                                                                "
        };
        let comment = spec.line_comments.first().map(|&lc| {
            if spec.lang == LangId::Cobol && lc == "*>" {
                "*> cmt_tail"
            } else {
                lc
            }
        });
        b.add_code_line(&[kw], pad, comment);

        let wrapped = format!("aa{kw}bb");
        b.add_code_line(&[&wrapped], "  ", None);

        let upper = kw.to_ascii_uppercase();
        b.add_code_line(&[&upper], "  ", None);
    }

    for &lc in spec.line_comments {
        let mut msg = String::from(first_kw);
        msg.push(' ');
        msg.push_str(second_kw);
        b.add_line_comment(lc, &msg);
        if spec.supports_spliced_line_comments && lc == "//" {
            b.add_spliced_line_comment(lc, first_kw);
        }
    }

    for &lsc in spec.line_start_comments {
        b.add_line_start_comment(lsc, first_kw);
    }

    for &(open, close, nested) in spec.block_comments {
        b.add_single_block_comment(open, close, first_kw);
        b.add_multi_block_comment(open, close, &[first_kw, "", "   ", second_kw]);
        b.add_mixed_block_and_code(open, close, first_kw);
        if nested {
            b.add_nested_block_comment(open, close, first_kw);
        }
    }

    for &(open, close) in spec.line_doc_blocks {
        b.add_line_doc_block(open, close, &[first_kw, "", "  \t", second_kw]);
    }

    for &(q, esc) in spec.quotes {
        b.add_quoted_line(q, esc, first_kw);
    }

    for &rq_spec in spec.raw_quotes {
        b.add_raw_quote(rq_spec, first_kw);
    }

    for &pref in spec.ident_prefixes {
        b.add_prefixed_keyword(pref, first_kw);
    }

    for &sig in spec.suppress_sigils {
        b.add_sigil_suppressed(sig, first_kw);
    }

    for &suf in spec.suppress_suffixes {
        if spec.has_ruby_defined_q && suf == "?" {
            b.buf.push_str("  defined?\n");
            b.count_keyword_token("defined");
            b.exp.lines += 1;
            b.exp.code += 1;
        } else {
            b.add_suffixed_suppressed(suf, first_kw);
        }
    }

    for &dc in spec.datum_comments {
        b.add_datum_comment(dc, first_kw);
    }

    if spec.supports_templates {
        b.add_template_interp(first_kw);
    }

    if spec.has_char_or_lifetime {
        let mut line = String::from("'a' '\\'' ");
        line.push_str(first_kw);
        b.buf.push_str("  ");
        b.buf.push_str(&line);
        b.buf.push('\n');
        b.count_keyword_token(first_kw);
        b.exp.lines += 1;
        b.exp.code += 1;
    }

    if spec.has_cpp_digit_sep {
        b.buf.push_str("  z9 = 1'000\n  ");
        b.buf.push_str(first_kw);
        b.buf.push('\n');
        b.count_keyword_token(first_kw);
        b.exp.lines += 2;
        b.exp.code += 2;
    }

    if spec.has_matlab_transpose {
        b.buf.push_str("  z9 = v0'\n  ");
        b.buf.push_str(first_kw);
        b.buf.push('\n');
        b.count_keyword_token(first_kw);
        b.exp.lines += 2;
        b.exp.code += 2;
    }

    if spec.has_tex_escaped_percent {
        b.buf.push_str("  50\\% ");
        b.buf.push_str(first_kw);
        b.buf.push('\n');
        b.count_keyword_token(first_kw);
        b.exp.lines += 1;
        b.exp.code += 1;
    }

    if spec.has_perl_hash_sigil {
        b.buf.push_str("  $#");
        b.buf.push_str(first_kw);
        b.buf.push('\n');
        b.exp.lines += 1;
        b.exp.code += 1;
    }

    if let Some(&eof) = spec.eof_markers.first() {
        b.add_eof_marker_tail(eof, first_kw);
    }

    b
}

#[test]
fn test_oracle_spec_completeness_and_file_detection() {
    assert_eq!(ORACLE_LANG_SPECS.len(), ALL_LANGS.len());
    assert_eq!(ORACLE_DOC_SPECS.len(), ALL_DOC_FORMATS.len());

    for (idx, spec) in ORACLE_LANG_SPECS.iter().enumerate() {
        assert_eq!(spec.lang, ALL_LANGS[idx]);
        if spec.lang != LangId::ObjectiveC {
            assert_eq!(
                detect_language(spec.sample_filename.as_bytes(), LANG_ALL_MASK),
                Some(spec.lang),
                "filename {} did not detect {}",
                spec.sample_filename,
                spec.lang.name()
            );
        } else {
            assert_eq!(
                detect_language(spec.sample_filename.as_bytes(), LANG_ALL_MASK),
                Some(LangId::Matlab)
            );
            assert!(objc::is_objc_file(
                b"@interface Foo : NSObject\n@end\n"
            ));
            assert!(!objc::is_objc_file(
                b"%{\n@interface comment\n%}\nfunction y = f(x)\ny = x;\nend\n"
            ));
        }

        let mut stats = LangStats::default();
        parse_into(spec.lang, b"  z9 = 42\n", &mut stats);
        assert_eq!(stats.lines(spec.lang), 1, "{}", spec.lang.name());
        assert_eq!(stats.code_lines(spec.lang), 1, "{}", spec.lang.name());
        assert_eq!(stats.comment_lines(spec.lang), 0, "{}", spec.lang.name());
        assert_eq!(stats.blank_lines(spec.lang), 0, "{}", spec.lang.name());
        assert_eq!(stats.total_keywords(spec.lang), 0, "{}", spec.lang.name());
    }

    for (idx, spec) in ORACLE_DOC_SPECS.iter().enumerate() {
        assert_eq!(spec.doc, ALL_DOC_FORMATS[idx]);
        assert_eq!(
            detect_doc_format(spec.sample_filename.as_bytes()),
            Some(spec.doc),
            "doc filename {} did not detect {}",
            spec.sample_filename,
            spec.doc.name()
        );
    }
}

#[test]
fn test_oracle_all_keywords_exhaustive_ground_truth() {
    for spec in ORACLE_LANG_SPECS {
        let kws = LangStats::keyword_names(spec.lang);
        for (slot, &kw) in kws.iter().enumerate() {
            let mut b = OracleBuilder::new(spec);
            b.add_code_line(&[kw], "  ", None);

            let padding_31 = "                               ";
            b.add_code_line(&[kw], padding_31, None);

            let upper = kw.to_ascii_uppercase();
            b.add_code_line(&[&upper], "  ", None);

            let wrapped = format!("p_{kw}_s");
            b.add_code_line(&[&wrapped], "  ", None);

            if let Some(&lc) = spec.line_comments.first() {
                b.add_line_comment(lc, kw);
            }
            if let Some(&(open, close, _)) = spec.block_comments.first() {
                b.add_single_block_comment(open, close, kw);
            }
            if let Some(&(q, esc)) = spec.quotes.first() {
                b.add_quoted_line(q, esc, kw);
            }
            for &sig in spec.suppress_sigils {
                b.add_sigil_suppressed(sig, kw);
            }
            for &pref in spec.ident_prefixes {
                b.add_prefixed_keyword(pref, kw);
            }
            let label = format!("kw[{}]={}", slot, kw);
            b.verify(&label);
        }
    }
}

#[test]
fn test_oracle_synthesized_torture_programs_ground_truth() {
    for spec in ORACLE_LANG_SPECS {
        let mut builder = build_torture_program(spec);
        builder.verify("torture_with_trailing_nl");

        if !builder.buf.ends_with('\n') {
            panic!("synthesized program must end with newline");
        }
        builder.buf.pop();
        builder.verify("torture_without_trailing_nl");

        let bytes = builder.buf.as_bytes();
        verify_line_invariants(spec.lang, bytes);
        for end in 0..=bytes.len() {
            verify_line_invariants(spec.lang, &bytes[..end]);
        }

        if let Some(&lc) = spec.line_comments.first() {
            let mut tail_cmt = OracleBuilder::new(spec);
            tail_cmt.buf.push_str(lc);
            tail_cmt.buf.push(' ');
            tail_cmt.buf.push_str(LangStats::keyword_names(spec.lang)[0]);
            tail_cmt.exp.lines = 1;
            tail_cmt.exp.comments = 1;
            tail_cmt.verify("eof_line_comment_no_nl");
            verify_line_invariants(spec.lang, tail_cmt.buf.as_bytes());
        }
    }

    for spec in ORACLE_DOC_SPECS {
        let mut src = String::new();
        let mut exp = DocStats::default();

        src.push('\n');
        src.push_str("   \t\r\n");
        exp.lines += 2;
        exp.blank_lines += 2;

        src.push_str(spec.neutral_line);
        src.push('\n');
        exp.lines += 1;
        exp.content_lines += 1;

        for &nc in spec.non_comment_directives {
            src.push_str(nc);
            src.push('\n');
            exp.lines += 1;
            exp.content_lines += 1;
            if nc.contains("<!--") && !nc.contains("-->") {
                src.push_str("  -->\n");
                exp.lines += 1;
                exp.comment_lines += 1;
            }
        }

        for &lc in spec.line_comments {
            src.push_str("  ");
            src.push_str(lc);
            src.push_str(" comment line\n");
            exp.lines += 1;
            exp.comment_lines += 1;
        }

        for &(open, close) in spec.block_comments {
            src.push_str("  ");
            src.push_str(open);
            src.push_str(" single ");
            src.push_str(close);
            src.push('\n');
            exp.lines += 1;
            exp.comment_lines += 1;

            src.push_str(open);
            src.push_str(" line1\n\nline3 ");
            src.push_str(close);
            src.push('\n');
            exp.lines += 3;
            exp.comment_lines += 3;

            src.push_str(open);
            src.push_str(" c1 ");
            src.push_str(close);
            src.push(' ');
            src.push_str(spec.neutral_line);
            src.push(' ');
            src.push_str(open);
            src.push_str(" c2 ");
            src.push_str(close);
            src.push('\n');
            exp.lines += 1;
            exp.content_lines += 1;
        }

        if spec.doc == DocFormatId::RestructuredText {
            src.push_str("..\n");
            src.push_str(".. single rst comment\n");
            src.push_str("..  \n");
            exp.lines += 3;
            exp.comment_lines += 3;
        }

        if spec.has_quotes_masking_comments {
            src.push_str("  k: 'http://ex /* a */'\n");
            exp.lines += 1;
            exp.content_lines += 1;
        }

        let mut actual = DocStats::default();
        parse_doc_into(spec.doc, src.as_bytes(), &mut actual);
        assert_eq!(actual, exp, "doc oracle failed for {}", spec.doc.name());
        assert_eq!(
            actual.lines,
            actual.content_lines + actual.comment_lines + actual.blank_lines,
            "doc sum failed for {}",
            spec.doc.name()
        );

        let crlf = src.replace("\r\n", "\n").replace('\n', "\r\n");
        let mut actual_crlf = DocStats::default();
        parse_doc_into(spec.doc, crlf.as_bytes(), &mut actual_crlf);
        assert_eq!(
            actual_crlf, exp,
            "doc CRLF oracle failed for {}",
            spec.doc.name()
        );

        let bytes = src.as_bytes();
        for end in 0..=bytes.len() {
            let mut t = DocStats::default();
            parse_doc_into(spec.doc, &bytes[..end], &mut t);
            assert_eq!(
                t.lines,
                t.content_lines + t.comment_lines + t.blank_lines,
                "doc truncation sum failed for {} at {}",
                spec.doc.name(),
                end
            );
        }
    }
}

#[test]
fn test_oracle_adversarial_boundaries_and_crlf() {
    for spec in ORACLE_LANG_SPECS {
        let mut torture = build_torture_program(spec);
        let crlf_buf = torture.buf.replace('\n', "\r\n");
        torture.buf = crlf_buf;
        torture.verify("torture_crlf");

        let kw = LangStats::keyword_names(spec.lang)[0];

        if let Some(&(open, close, nested)) = spec.block_comments.first() {
            let mut big_cmt = OracleBuilder::new(spec);
            big_cmt.buf.push_str(open);
            big_cmt.buf.push('\n');
            big_cmt.exp.lines += 1;
            big_cmt.exp.comments += 1;
            for n in 0..300 {
                if n % 17 == 0 {
                    big_cmt.buf.push('\n');
                } else {
                    big_cmt.buf.push_str("  0123456789abcdef0123456789 ");
                    big_cmt.buf.push_str(kw);
                    big_cmt.buf.push('\n');
                }
                big_cmt.exp.lines += 1;
                big_cmt.exp.comments += 1;
            }
            big_cmt.buf.push_str(close);
            big_cmt.buf.push('\n');
            big_cmt.exp.lines += 1;
            big_cmt.exp.comments += 1;
            big_cmt.add_code_line(&[kw], "  ", None);
            big_cmt.verify("large_8500b_block_comment");

            let mut same_line = OracleBuilder::new(spec);
            same_line.buf.push_str("  ");
            same_line.buf.push_str(open);
            same_line.buf.push_str(kw);
            same_line.buf.push_str(close);
            same_line.buf.push(' ');
            same_line.buf.push_str(open);
            same_line.buf.push_str(kw);
            same_line.buf.push_str(close);
            same_line.buf.push('\n');
            same_line.exp.lines += 1;
            same_line.exp.comments += 1;
            same_line.verify("two_blocks_comment_only_line");

            let mut sand = OracleBuilder::new(spec);
            sand.buf.push_str("  ");
            sand.buf.push_str(open);
            sand.buf.push_str(kw);
            sand.buf.push_str(close);
            sand.buf.push(' ');
            sand.buf.push_str(kw);
            sand.buf.push(' ');
            sand.buf.push_str(open);
            sand.buf.push_str(kw);
            sand.buf.push_str(close);
            sand.buf.push(' ');
            sand.buf.push_str(kw);
            sand.buf.push('\n');
            sand.count_keyword_token(kw);
            sand.count_keyword_token(kw);
            sand.exp.lines += 1;
            sand.exp.code += 1;
            sand.verify("code_between_two_blocks");

            if nested {
                let mut d3 = OracleBuilder::new(spec);
                d3.buf.push_str(open);
                d3.buf.push(' ');
                d3.buf.push_str(kw);
                d3.buf.push('\n');
                d3.buf.push_str(open);
                d3.buf.push(' ');
                d3.buf.push_str(open);
                d3.buf.push(' ');
                d3.buf.push_str(kw);
                d3.buf.push(' ');
                d3.buf.push_str(close);
                d3.buf.push(' ');
                d3.buf.push_str(kw);
                d3.buf.push(' ');
                d3.buf.push_str(close);
                d3.buf.push('\n');
                d3.buf.push_str(kw);
                d3.buf.push(' ');
                d3.buf.push_str(close);
                d3.buf.push('\n');
                d3.exp.lines += 3;
                d3.exp.comments += 3;
                d3.add_code_line(&[kw], "  ", None);
                d3.verify("depth3_nested_block");
            }
        }

        if let Some(&(q, _)) = spec.quotes.first() {
            let mut big_str = OracleBuilder::new(spec);
            big_str.buf.push_str("  ");
            big_str.buf.push_str(q);
            for _ in 0..300 {
                big_str.buf.push_str("0123456789abcdef0123456789 ");
                big_str.buf.push_str(kw);
                big_str.buf.push('\n');
                big_str.exp.lines += 1;
                big_str.exp.code += 1;
            }
            big_str.buf.push_str(q);
            big_str.buf.push('\n');
            big_str.exp.lines += 1;
            big_str.exp.code += 1;
            big_str.verify("large_8500b_quoted_literal");
        }

        let mut lone_toks: Vec<&str> = Vec::new();
        for &(open, _, _) in spec.block_comments {
            let first = &open[..1];
            if open.len() >= 2
                && !spec.line_comments.contains(&first)
                && !spec.block_comments.iter().any(|&(o, _, _)| o == first)
                && !lone_toks.contains(&first)
            {
                lone_toks.push(first);
            }
        }
        for &lc in spec.line_comments {
            let first = &lc[..1];
            if lc.len() >= 2
                && !spec.line_comments.contains(&first)
                && !spec.block_comments.iter().any(|&(o, _, _)| o == first)
                && !lone_toks.contains(&first)
            {
                lone_toks.push(first);
            }
        }
        for rq in spec.raw_quotes {
            let first = &rq.open[..1];
            if (rq.open.starts_with('r') || rq.open.starts_with('{') || rq.open.starts_with('['))
                && !spec.block_comments.iter().any(|&(o, _, _)| o == first)
                && !lone_toks.contains(&first)
            {
                lone_toks.push(first);
            }
        }
        if spec.lang == LangId::Perl {
            lone_toks.push("$#");
            lone_toks.push("=1");
        }
        if spec.lang == LangId::Ruby {
            lone_toks.push("=b");
        }
        if spec.lang == LangId::Dart || spec.lang == LangId::Scala || spec.lang == LangId::Rust {
            lone_toks.push("r");
        }
        for &tok in &lone_toks {
            if spec.lang == LangId::Scheme && tok == "/" {
                continue;
            }
            let mut single = OracleBuilder::new(spec);
            single.buf.push_str(tok);
            single.exp.lines = 1;
            single.exp.code = 1;
            single.verify("lone_prefix_eof");

            single.buf.push('\n');
            single.verify("lone_prefix_nl");
        }
    }
}

#[test]
fn test_oracle_adversarial_interactions() {
    for spec in ORACLE_LANG_SPECS {
        let kws = LangStats::keyword_names(spec.lang);
        let kw = kws[0];
        let kw2 = kws[kws.len() - 1];

        let mut no_space_seq = OracleBuilder::new(spec);
        for (i, &k) in kws.iter().enumerate() {
            if i > 0 {
                no_space_seq.buf.push(',');
            }
            no_space_seq.buf.push_str(k);
            no_space_seq.count_keyword_token(k);
        }
        no_space_seq.exp.lines = 1;
        no_space_seq.exp.code = 1;
        no_space_seq.verify("comma_separated_all_keywords_eof");
        no_space_seq.buf.push('\n');
        no_space_seq.verify("comma_separated_all_keywords_nl");

        if let Some(&lc) = spec.line_comments.first() {
            let mut trap = OracleBuilder::new(spec);
            if let Some(&(q, _)) = spec.quotes.first() {
                trap.buf.push_str("  ");
                trap.buf.push_str(q);
                trap.buf.push_str(lc);
                trap.buf.push(' ');
                trap.buf.push_str(kw);
                trap.buf.push_str(q);
                trap.buf.push('\n');
                trap.exp.lines += 1;
                trap.exp.code += 1;

                trap.buf.push_str(lc);
                trap.buf.push(' ');
                trap.buf.push_str(q);
                trap.buf.push_str(" unclosed_quote ");
                trap.buf.push_str(kw);
                trap.buf.push('\n');
                trap.exp.lines += 1;
                trap.exp.comments += 1;
            }
            trap.add_code_line(&[kw2], "  ", None);
            trap.verify("line_comment_quote_cross_trap");
        }

        for &(open, close, nested) in spec.block_comments {
            let mut tightly_packed = OracleBuilder::new(spec);
            tightly_packed.buf.push_str(open);
            tightly_packed.buf.push_str(kw);
            tightly_packed.buf.push_str(close);
            tightly_packed.buf.push_str(kw);
            tightly_packed.count_keyword_token(kw);
            tightly_packed.buf.push('\n');

            tightly_packed.buf.push_str("  ");
            tightly_packed.buf.push_str(kw2);
            tightly_packed.count_keyword_token(kw2);
            tightly_packed.buf.push_str(open);
            tightly_packed.buf.push_str(kw);
            tightly_packed.buf.push_str(close);
            tightly_packed.buf.push('\n');
            tightly_packed.exp.lines = 2;
            tightly_packed.exp.code = 2;
            tightly_packed.verify("tight_block_comment_adjacent_kw");

            if let Some(&(q, esc)) = spec.quotes.first() {
                let mut q_trap = OracleBuilder::new(spec);
                q_trap.buf.push_str("  ");
                q_trap.buf.push_str(q);
                if esc {
                    q_trap.buf.push('\\');
                    q_trap.buf.push_str(q);
                }
                q_trap.buf.push_str(open);
                q_trap.buf.push(' ');
                q_trap.buf.push_str(kw);
                q_trap.buf.push_str(q);
                q_trap.buf.push('\n');
                q_trap.exp.lines += 1;
                q_trap.exp.code += 1;

                q_trap.buf.push_str("  ");
                q_trap.buf.push_str(open);
                q_trap.buf.push(' ');
                q_trap.buf.push_str(q);
                q_trap.buf.push(' ');
                q_trap.buf.push_str(kw);
                q_trap.buf.push(' ');
                q_trap.buf.push_str(close);
                q_trap.buf.push('\n');
                q_trap.exp.lines += 1;
                q_trap.exp.comments += 1;

                q_trap.add_code_line(&[kw2], "  ", None);
                q_trap.verify("block_comment_quote_cross_trap");
            }

            let mut unclosed_blank_tail = OracleBuilder::new(spec);
            unclosed_blank_tail.buf.push_str(open);
            unclosed_blank_tail.buf.push(' ');
            unclosed_blank_tail.buf.push_str(kw);
            unclosed_blank_tail.buf.push_str("\n   ");
            unclosed_blank_tail.exp.lines = 2;
            unclosed_blank_tail.exp.comments = 2;
            unclosed_blank_tail.verify("unclosed_block_blank_tail_eof");

            let mut mixed_unclosed = OracleBuilder::new(spec);
            mixed_unclosed.buf.push_str("  ");
            mixed_unclosed.buf.push_str(kw);
            mixed_unclosed.count_keyword_token(kw);
            mixed_unclosed.buf.push(' ');
            mixed_unclosed.buf.push_str(open);
            mixed_unclosed.buf.push('\n');
            mixed_unclosed.buf.push_str(kw2);
            mixed_unclosed.exp.lines = 2;
            mixed_unclosed.exp.code = 1;
            mixed_unclosed.exp.comments = 1;
            mixed_unclosed.verify("mixed_unclosed_block_eof");

            if nested {
                let mut nested_unclosed = OracleBuilder::new(spec);
                nested_unclosed.buf.push_str(open);
                nested_unclosed.buf.push(' ');
                nested_unclosed.buf.push_str(kw);
                nested_unclosed.buf.push('\n');
                nested_unclosed.buf.push_str(open);
                nested_unclosed.buf.push(' ');
                nested_unclosed.buf.push_str(close);
                nested_unclosed.buf.push('\n');
                nested_unclosed.buf.push_str(kw2);
                nested_unclosed.exp.lines = 3;
                nested_unclosed.exp.comments = 3;
                nested_unclosed.verify("nested_unclosed_outer_eof");
            }
        }
    }

    for spec in ORACLE_DOC_SPECS {
        if let Some(&(open, close)) = spec.block_comments.first() {
            let mut src = String::new();
            src.push_str(open);
            src.push_str(" a ");
            src.push_str(close);
            src.push(' ');
            src.push_str(open);
            src.push_str(" b ");
            src.push_str(close);
            src.push('\n');

            src.push_str(open);
            src.push_str(" a ");
            src.push_str(close);
            src.push(' ');
            src.push_str(spec.neutral_line);
            src.push(' ');
            src.push_str(open);
            src.push('\n');
            src.push_str(spec.neutral_line);
            src.push('\n');
            src.push_str(close);
            src.push('\n');

            let mut stats = DocStats::default();
            parse_doc_into(spec.doc, src.as_bytes(), &mut stats);
            assert_eq!(stats.lines, 4, "{}", spec.doc.name());
            assert_eq!(stats.content_lines, 1, "{}", spec.doc.name());
            assert_eq!(stats.comment_lines, 3, "{}", spec.doc.name());
            assert_eq!(stats.blank_lines, 0, "{}", spec.doc.name());
        }
    }
}
