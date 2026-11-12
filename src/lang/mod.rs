pub mod ada;
pub mod assembly;
pub mod awk;
pub mod bash;
pub mod c;
pub mod clisp;
pub mod clojure;
pub mod cmake;
pub mod cobol;
pub mod cpp;
pub mod csharp;
pub mod css;
pub mod cuda;
pub mod d;
pub mod dart;
pub mod dockerfile;
pub mod elixir;
pub mod erlang;
pub mod fish;
pub mod fortran;
pub mod fsharp;
pub mod gleam;
pub mod glsl;
pub mod go;
pub mod graphql;
pub mod groovy;
pub mod hare;
pub mod haskell;
pub mod hlsl;
pub mod html;
pub mod java;
pub mod javascript;
pub mod json;
pub mod julia;
pub mod kotlin;
pub mod lua;
pub mod makefile;
pub mod matlab;
pub mod mojo;
pub mod nim;
pub mod objc;
pub mod ocaml;
pub mod odin;
pub mod pascal;
pub mod perl;
pub mod php;
pub mod powershell;
pub mod protobuf;
pub mod python;
pub mod r;
pub mod ruby;
pub mod rust;
pub mod scala;
pub mod scheme;
pub mod solidity;
pub mod sql;
pub mod swift;
pub mod tcl;
pub mod toml;
pub mod typescript;
pub mod v;
pub mod verilog;
pub mod vhdl;
pub mod wgsl;
pub mod xml;
pub mod yaml;
pub mod zig;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LangId {
    C = 0,
    Cpp = 1,
    Rust = 2,
    Python = 3,
    Perl = 4,
    Go = 5,
    Zig = 6,
    Odin = 7,
    JavaScript = 8,
    TypeScript = 9,
    Lua = 10,
    Java = 11,
    CSharp = 12,
    Ruby = 13,
    Php = 14,
    Swift = 15,
    Kotlin = 16,
    Bash = 17,
    Sql = 18,
    Dart = 19,
    Scala = 20,
    Haskell = 21,
    OCaml = 22,
    Gleam = 23,
    D = 24,
    Nim = 25,
    Fortran = 26,
    Elixir = 27,
    Erlang = 28,
    Julia = 29,
    R = 30,
    Matlab = 31,
    Solidity = 32,
    FSharp = 33,
    Mojo = 34,
    Verilog = 35,
    Cuda = 36,
    Wgsl = 37,
    Ada = 38,
    Groovy = 39,
    Clojure = 40,
    Assembly = 41,
    Tcl = 42,
    Scheme = 43,
    Json = 44,
    Toml = 45,
    Yaml = 46,
    Dockerfile = 47,
    Makefile = 48,
    CMake = 49,
    Glsl = 50,
    Hlsl = 51,
    ObjectiveC = 52,
    V = 53,
    CommonLisp = 54,
    Pascal = 55,
    PowerShell = 56,
    Fish = 57,
    Awk = 58,
    Hare = 59,
    Html = 60,
    Xml = 61,
    Css = 62,
    Graphql = 63,
    Protobuf = 64,
    Cobol = 65,
    Vhdl = 66,
    CHeader = 67,
    CppHeader = 68,
}

pub const LANG_COUNT: usize = 69;
pub const LANG_C_MASK: u128 = 1 << (LangId::C as u128);
pub const LANG_CPP_MASK: u128 = 1 << (LangId::Cpp as u128);
pub const LANG_RUST_MASK: u128 = 1 << (LangId::Rust as u128);
pub const LANG_PYTHON_MASK: u128 = 1 << (LangId::Python as u128);
pub const LANG_PERL_MASK: u128 = 1 << (LangId::Perl as u128);
pub const LANG_GO_MASK: u128 = 1 << (LangId::Go as u128);
pub const LANG_ZIG_MASK: u128 = 1 << (LangId::Zig as u128);
pub const LANG_ODIN_MASK: u128 = 1 << (LangId::Odin as u128);
pub const LANG_JAVASCRIPT_MASK: u128 = 1 << (LangId::JavaScript as u128);
pub const LANG_TYPESCRIPT_MASK: u128 = 1 << (LangId::TypeScript as u128);
pub const LANG_LUA_MASK: u128 = 1 << (LangId::Lua as u128);
pub const LANG_JAVA_MASK: u128 = 1 << (LangId::Java as u128);
pub const LANG_CSHARP_MASK: u128 = 1 << (LangId::CSharp as u128);
pub const LANG_RUBY_MASK: u128 = 1 << (LangId::Ruby as u128);
pub const LANG_PHP_MASK: u128 = 1 << (LangId::Php as u128);
pub const LANG_SWIFT_MASK: u128 = 1 << (LangId::Swift as u128);
pub const LANG_KOTLIN_MASK: u128 = 1 << (LangId::Kotlin as u128);
pub const LANG_BASH_MASK: u128 = 1 << (LangId::Bash as u128);
pub const LANG_SQL_MASK: u128 = 1 << (LangId::Sql as u128);
pub const LANG_DART_MASK: u128 = 1 << (LangId::Dart as u128);
pub const LANG_SCALA_MASK: u128 = 1 << (LangId::Scala as u128);
pub const LANG_HASKELL_MASK: u128 = 1 << (LangId::Haskell as u128);
pub const LANG_OCAML_MASK: u128 = 1 << (LangId::OCaml as u128);
pub const LANG_GLEAM_MASK: u128 = 1 << (LangId::Gleam as u128);
pub const LANG_D_MASK: u128 = 1 << (LangId::D as u128);
pub const LANG_NIM_MASK: u128 = 1 << (LangId::Nim as u128);
pub const LANG_FORTRAN_MASK: u128 = 1 << (LangId::Fortran as u128);
pub const LANG_ELIXIR_MASK: u128 = 1 << (LangId::Elixir as u128);
pub const LANG_ERLANG_MASK: u128 = 1 << (LangId::Erlang as u128);
pub const LANG_JULIA_MASK: u128 = 1 << (LangId::Julia as u128);
pub const LANG_R_MASK: u128 = 1 << (LangId::R as u128);
pub const LANG_MATLAB_MASK: u128 = 1 << (LangId::Matlab as u128);
pub const LANG_SOLIDITY_MASK: u128 = 1 << (LangId::Solidity as u128);
pub const LANG_FSHARP_MASK: u128 = 1 << (LangId::FSharp as u128);
pub const LANG_MOJO_MASK: u128 = 1 << (LangId::Mojo as u128);
pub const LANG_VERILOG_MASK: u128 = 1 << (LangId::Verilog as u128);
pub const LANG_CUDA_MASK: u128 = 1 << (LangId::Cuda as u128);
pub const LANG_WGSL_MASK: u128 = 1 << (LangId::Wgsl as u128);
pub const LANG_ADA_MASK: u128 = 1 << (LangId::Ada as u128);
pub const LANG_GROOVY_MASK: u128 = 1 << (LangId::Groovy as u128);
pub const LANG_CLOJURE_MASK: u128 = 1 << (LangId::Clojure as u128);
pub const LANG_ASSEMBLY_MASK: u128 = 1 << (LangId::Assembly as u128);
pub const LANG_TCL_MASK: u128 = 1 << (LangId::Tcl as u128);
pub const LANG_SCHEME_MASK: u128 = 1 << (LangId::Scheme as u128);
pub const LANG_JSON_MASK: u128 = 1 << (LangId::Json as u128);
pub const LANG_TOML_MASK: u128 = 1 << (LangId::Toml as u128);
pub const LANG_YAML_MASK: u128 = 1 << (LangId::Yaml as u128);
pub const LANG_DOCKERFILE_MASK: u128 = 1 << (LangId::Dockerfile as u128);
pub const LANG_MAKEFILE_MASK: u128 = 1 << (LangId::Makefile as u128);
pub const LANG_CMAKE_MASK: u128 = 1 << (LangId::CMake as u128);
pub const LANG_GLSL_MASK: u128 = 1 << (LangId::Glsl as u128);
pub const LANG_HLSL_MASK: u128 = 1 << (LangId::Hlsl as u128);
pub const LANG_OBJECTIVEC_MASK: u128 = 1 << (LangId::ObjectiveC as u128);
pub const LANG_V_MASK: u128 = 1 << (LangId::V as u128);
pub const LANG_COMMONLISP_MASK: u128 = 1 << (LangId::CommonLisp as u128);
pub const LANG_PASCAL_MASK: u128 = 1 << (LangId::Pascal as u128);
pub const LANG_POWERSHELL_MASK: u128 = 1 << (LangId::PowerShell as u128);
pub const LANG_FISH_MASK: u128 = 1 << (LangId::Fish as u128);
pub const LANG_AWK_MASK: u128 = 1 << (LangId::Awk as u128);
pub const LANG_HARE_MASK: u128 = 1 << (LangId::Hare as u128);
pub const LANG_HTML_MASK: u128 = 1 << (LangId::Html as u128);
pub const LANG_XML_MASK: u128 = 1 << (LangId::Xml as u128);
pub const LANG_CSS_MASK: u128 = 1 << (LangId::Css as u128);
pub const LANG_GRAPHQL_MASK: u128 = 1 << (LangId::Graphql as u128);
pub const LANG_PROTOBUF_MASK: u128 = 1 << (LangId::Protobuf as u128);
pub const LANG_COBOL_MASK: u128 = 1 << (LangId::Cobol as u128);
pub const LANG_VHDL_MASK: u128 = 1 << (LangId::Vhdl as u128);
pub const LANG_CHEADER_MASK: u128 = 1 << (LangId::CHeader as u128);
pub const LANG_CPPHEADER_MASK: u128 = 1 << (LangId::CppHeader as u128);
pub const LANG_ALL_MASK: u128 = if LANG_COUNT == 128 {
    !0u128
} else {
    (1u128 << LANG_COUNT) - 1
};

pub const ALL_LANGS: [LangId; LANG_COUNT] = [
    LangId::C,
    LangId::Cpp,
    LangId::Rust,
    LangId::Python,
    LangId::Perl,
    LangId::Go,
    LangId::Zig,
    LangId::Odin,
    LangId::JavaScript,
    LangId::TypeScript,
    LangId::Lua,
    LangId::Java,
    LangId::CSharp,
    LangId::Ruby,
    LangId::Php,
    LangId::Swift,
    LangId::Kotlin,
    LangId::Bash,
    LangId::Sql,
    LangId::Dart,
    LangId::Scala,
    LangId::Haskell,
    LangId::OCaml,
    LangId::Gleam,
    LangId::D,
    LangId::Nim,
    LangId::Fortran,
    LangId::Elixir,
    LangId::Erlang,
    LangId::Julia,
    LangId::R,
    LangId::Matlab,
    LangId::Solidity,
    LangId::FSharp,
    LangId::Mojo,
    LangId::Verilog,
    LangId::Cuda,
    LangId::Wgsl,
    LangId::Ada,
    LangId::Groovy,
    LangId::Clojure,
    LangId::Assembly,
    LangId::Tcl,
    LangId::Scheme,
    LangId::Json,
    LangId::Toml,
    LangId::Yaml,
    LangId::Dockerfile,
    LangId::Makefile,
    LangId::CMake,
    LangId::Glsl,
    LangId::Hlsl,
    LangId::ObjectiveC,
    LangId::V,
    LangId::CommonLisp,
    LangId::Pascal,
    LangId::PowerShell,
    LangId::Fish,
    LangId::Awk,
    LangId::Hare,
    LangId::Html,
    LangId::Xml,
    LangId::Css,
    LangId::Graphql,
    LangId::Protobuf,
    LangId::Cobol,
    LangId::Vhdl,
    LangId::CHeader,
    LangId::CppHeader,
];

const _: () = assert!(LANG_COUNT > 0 && LANG_COUNT <= 128);
const _: () = assert!(ALL_LANGS.len() == LANG_COUNT);
const _: () = {
    let mut i = 0;
    while i < LANG_COUNT {
        assert!(ALL_LANGS[i] as usize == i);
        i += 1;
    }
};

impl LangId {
    #[inline(always)]
    pub const fn name(self) -> &'static str {
        match self {
            Self::C => "C",
            Self::Cpp => "C++",
            Self::Rust => "Rust",
            Self::Python => "Python",
            Self::Perl => "Perl",
            Self::Go => "Go",
            Self::Zig => "Zig",
            Self::Odin => "Odin",
            Self::JavaScript => "JavaScript",
            Self::TypeScript => "TypeScript",
            Self::Lua => "Lua",
            Self::Java => "Java",
            Self::CSharp => "C#",
            Self::Ruby => "Ruby",
            Self::Php => "PHP",
            Self::Swift => "Swift",
            Self::Kotlin => "Kotlin",
            Self::Bash => "Bash",
            Self::Sql => "SQL",
            Self::Dart => "Dart",
            Self::Scala => "Scala",
            Self::Haskell => "Haskell",
            Self::OCaml => "OCaml",
            Self::Gleam => "Gleam",
            Self::D => "D",
            Self::Nim => "Nim",
            Self::Fortran => "Fortran",
            Self::Elixir => "Elixir",
            Self::Erlang => "Erlang",
            Self::Julia => "Julia",
            Self::R => "R",
            Self::Matlab => "MATLAB",
            Self::Solidity => "Solidity",
            Self::FSharp => "F#",
            Self::Mojo => "Mojo",
            Self::Verilog => "Verilog",
            Self::Cuda => "CUDA",
            Self::Wgsl => "WGSL",
            Self::Ada => "Ada",
            Self::Groovy => "Groovy",
            Self::Clojure => "Clojure",
            Self::Assembly => "Assembly",
            Self::Tcl => "Tcl",
            Self::Scheme => "Scheme",
            Self::Json => "JSON",
            Self::Toml => "TOML",
            Self::Yaml => "YAML",
            Self::Dockerfile => "Dockerfile",
            Self::Makefile => "Makefile",
            Self::CMake => "CMake",
            Self::Glsl => "GLSL",
            Self::Hlsl => "HLSL",
            Self::ObjectiveC => "Objective-C",
            Self::V => "V",
            Self::CommonLisp => "Common Lisp",
            Self::Pascal => "Pascal",
            Self::PowerShell => "PowerShell",
            Self::Fish => "Fish",
            Self::Awk => "AWK",
            Self::Hare => "Hare",
            Self::Html => "HTML",
            Self::Xml => "XML",
            Self::Css => "CSS",
            Self::Graphql => "GraphQL",
            Self::Protobuf => "Protobuf",
            Self::Cobol => "COBOL",
            Self::Vhdl => "VHDL",
            Self::CHeader => "C Header",
            Self::CppHeader => "C++ Header",
        }
    }

    #[inline(always)]
    pub const fn mask(self) -> u128 {
        1 << (self as u128)
    }

    #[inline(always)]
    pub const fn flag_aliases(self) -> &'static [&'static str] {
        match self {
            Self::C => &["-c", "--c"],
            Self::Cpp => &["-cpp", "--cpp", "-c++", "--c++"],
            Self::Rust => &["-rs", "--rs", "-rust", "--rust"],
            Self::Python => &["-py", "--py", "-python", "--python"],
            Self::Perl => &["-pl", "--pl", "-perl", "--perl"],
            Self::Go => &["-go", "--go", "-golang", "--golang"],
            Self::Zig => &["-zig", "--zig"],
            Self::Odin => &["-odin", "--odin"],
            Self::JavaScript => &["-js", "--js", "-javascript", "--javascript"],
            Self::TypeScript => &["-ts", "--ts", "-typescript", "--typescript"],
            Self::Lua => &["-lua", "--lua"],
            Self::Java => &["-java", "--java"],
            Self::CSharp => &["-cs", "--cs", "-csharp", "--csharp", "-c#", "--c#"],
            Self::Ruby => &["-rb", "--rb", "-ruby", "--ruby"],
            Self::Php => &["-php", "--php"],
            Self::Swift => &["-swift", "--swift"],
            Self::Kotlin => &["-kt", "--kt", "-kotlin", "--kotlin"],
            Self::Bash => &["-sh", "--sh", "-bash", "--bash"],
            Self::Sql => &["-sql", "--sql"],
            Self::Dart => &["-dart", "--dart"],
            Self::Scala => &["-scala", "--scala"],
            Self::Haskell => &["-hs", "--hs", "-haskell", "--haskell"],
            Self::OCaml => &["-ml", "--ml", "-ocaml", "--ocaml"],
            Self::Gleam => &["-gleam", "--gleam"],
            Self::D => &["-d", "--d"],
            Self::Nim => &["-nim", "--nim"],
            Self::Fortran => &["-f", "--f", "-fortran", "--fortran"],
            Self::Elixir => &["-ex", "--ex", "-elixir", "--elixir"],
            Self::Erlang => &["-erl", "--erl", "-erlang", "--erlang"],
            Self::Julia => &["-jl", "--jl", "-julia", "--julia"],
            Self::R => &["-r", "--r"],
            Self::Matlab => &["-m", "--m", "-matlab", "--matlab", "-octave", "--octave"],
            Self::Solidity => &["-sol", "--sol", "-solidity", "--solidity"],
            Self::FSharp => &["-fs", "--fs", "-fsharp", "--fsharp", "-f#", "--f#"],
            Self::Mojo => &["-mojo", "--mojo"],
            Self::Verilog => &["-v", "--v", "-verilog", "--verilog", "-sv", "--sv"],
            Self::Cuda => &["-cu", "--cu", "-cuda", "--cuda"],
            Self::Wgsl => &["-wgsl", "--wgsl"],
            Self::Ada => &["-ada", "--ada"],
            Self::Groovy => &["-groovy", "--groovy", "-gvy", "--gvy"],
            Self::Clojure => &["-clj", "--clj", "-clojure", "--clojure"],
            Self::Assembly => &["-asm", "--asm", "-s", "--s", "-assembly", "--assembly"],
            Self::Tcl => &["-tcl", "--tcl"],
            Self::Scheme => &["-scm", "--scm", "-scheme", "--scheme", "-rkt", "--rkt"],
            Self::Json => &["-json", "--json-lang"],
            Self::Toml => &["-toml", "--toml"],
            Self::Yaml => &["-yaml", "--yaml", "-yml", "--yml"],
            Self::Dockerfile => &["-docker", "--docker", "-dockerfile", "--dockerfile"],
            Self::Makefile => &["-make", "--make", "-makefile", "--makefile"],
            Self::CMake => &["-cmake", "--cmake"],
            Self::Glsl => &["-glsl", "--glsl"],
            Self::Hlsl => &["-hlsl", "--hlsl"],
            Self::ObjectiveC => &["-objc", "--objc"],
            Self::V => &["-vlang", "--vlang"],
            Self::CommonLisp => &["-lisp", "--lisp", "-clisp", "--clisp", "-common-lisp", "--common-lisp"],
            Self::Pascal => &["-pas", "--pas", "-pascal", "--pascal"],
            Self::PowerShell => &["-ps1", "--ps1", "-powershell", "--powershell", "-pwsh", "--pwsh"],
            Self::Fish => &["-fish", "--fish"],
            Self::Awk => &["-awk", "--awk"],
            Self::Hare => &["-ha", "--ha", "-hare", "--hare"],
            Self::Html => &["-html", "--html", "-htm", "--htm"],
            Self::Xml => &["-xml", "--xml"],
            Self::Css => &["-css", "--css", "-scss", "--scss", "-sass", "--sass", "-less", "--less"],
            Self::Graphql => &["-graphql", "--graphql", "-gql", "--gql"],
            Self::Protobuf => &["-proto", "--proto", "-protobuf", "--protobuf"],
            Self::Cobol => &["-cobol", "--cobol", "-cbl", "--cbl"],
            Self::Vhdl => &["-vhdl", "--vhdl", "-vhd", "--vhd"],
            Self::CHeader => &["-ch", "--ch", "-c-header", "--c-header"],
            Self::CppHeader => &["-cxxh", "--cxxh", "-cpp-header", "--cpp-header"],
        }
    }

    #[inline(always)]
    pub fn from_flag(arg: &[u8]) -> Option<Self> {
        match arg {
            b"-c" | b"--c" | b"-C" => Some(Self::C),
            b"-cpp" | b"--cpp" | b"-CPP" | b"--CPP" | b"-c++" | b"--c++" | b"-C++" | b"--C++" => {
                Some(Self::Cpp)
            }
            b"-rs" | b"--rs" | b"-RS" | b"--RS" | b"-rust" | b"--rust" | b"-RUST" | b"--RUST" => {
                Some(Self::Rust)
            }
            b"-py" | b"--py" | b"-PY" | b"--PY" | b"-python" | b"--python" | b"-PYTHON"
            | b"--PYTHON" => Some(Self::Python),
            b"-pl" | b"--pl" | b"-PL" | b"--PL" | b"-perl" | b"--perl" | b"-PERL" | b"--PERL" => {
                Some(Self::Perl)
            }
            b"-go" | b"--go" | b"-GO" | b"--GO" | b"-golang" | b"--golang" | b"-GOLANG"
            | b"--GOLANG" => Some(Self::Go),
            b"-zig" | b"--zig" | b"-ZIG" | b"--ZIG" => Some(Self::Zig),
            b"-odin" | b"--odin" | b"-ODIN" | b"--ODIN" => Some(Self::Odin),
            b"-js" | b"--js" | b"-JS" | b"--JS" | b"-javascript" | b"--javascript"
            | b"-JAVASCRIPT" | b"--JAVASCRIPT" => Some(Self::JavaScript),
            b"-ts" | b"--ts" | b"-TS" | b"--TS" | b"-typescript" | b"--typescript"
            | b"-TYPESCRIPT" | b"--TYPESCRIPT" => Some(Self::TypeScript),
            b"-lua" | b"--lua" | b"-LUA" | b"--LUA" => Some(Self::Lua),
            b"-java" | b"--java" | b"-JAVA" | b"--JAVA" => Some(Self::Java),
            b"-cs" | b"--cs" | b"-CS" | b"--CS" | b"-csharp" | b"--csharp" | b"-CSHARP"
            | b"--CSHARP" | b"-c#" | b"--c#" | b"-C#" | b"--C#" => Some(Self::CSharp),
            b"-rb" | b"--rb" | b"-RB" | b"--RB" | b"-ruby" | b"--ruby" | b"-RUBY" | b"--RUBY" => {
                Some(Self::Ruby)
            }
            b"-php" | b"--php" | b"-PHP" | b"--PHP" => Some(Self::Php),
            b"-swift" | b"--swift" | b"-SWIFT" | b"--SWIFT" => Some(Self::Swift),
            b"-kt" | b"--kt" | b"-KT" | b"--KT" | b"-kotlin" | b"--kotlin" | b"-KOTLIN"
            | b"--KOTLIN" => Some(Self::Kotlin),
            b"-sh" | b"--sh" | b"-SH" | b"--SH" | b"-bash" | b"--bash" | b"-BASH" | b"--BASH" => {
                Some(Self::Bash)
            }
            b"-sql" | b"--sql" | b"-SQL" | b"--SQL" => Some(Self::Sql),
            b"-dart" | b"--dart" | b"-DART" | b"--DART" => Some(Self::Dart),
            b"-scala" | b"--scala" | b"-SCALA" | b"--SCALA" => Some(Self::Scala),
            b"-hs" | b"--hs" | b"-HS" | b"--HS" | b"-haskell" | b"--haskell" | b"-HASKELL"
            | b"--HASKELL" => Some(Self::Haskell),
            b"-ml" | b"--ml" | b"-ML" | b"--ML" | b"-ocaml" | b"--ocaml" | b"-OCAML"
            | b"--OCAML" => Some(Self::OCaml),
            b"-gleam" | b"--gleam" | b"-GLEAM" | b"--GLEAM" => Some(Self::Gleam),
            b"-d" | b"--d" | b"-D" => Some(Self::D),
            b"-nim" | b"--nim" | b"-NIM" | b"--NIM" => Some(Self::Nim),
            b"-f" | b"--f" | b"-F" | b"--F" | b"-fortran" | b"--fortran" | b"-FORTRAN"
            | b"--FORTRAN" => Some(Self::Fortran),
            b"-ex" | b"--ex" | b"-EX" | b"--EX" | b"-elixir" | b"--elixir" | b"-ELIXIR"
            | b"--ELIXIR" => Some(Self::Elixir),
            b"-erl" | b"--erl" | b"-ERL" | b"--ERL" | b"-erlang" | b"--erlang" | b"-ERLANG"
            | b"--ERLANG" => Some(Self::Erlang),
            b"-jl" | b"--jl" | b"-JL" | b"--JL" | b"-julia" | b"--julia" | b"-JULIA"
            | b"--JULIA" => Some(Self::Julia),
            b"-r" | b"--r" | b"-R" | b"--R" => Some(Self::R),
            b"-m" | b"--m" | b"-M" | b"--M" | b"-matlab" | b"--matlab" | b"-MATLAB"
            | b"--MATLAB" | b"-octave" | b"--octave" | b"-OCTAVE" | b"--OCTAVE" => {
                Some(Self::Matlab)
            }
            b"-sol" | b"--sol" | b"-SOL" | b"--SOL" | b"-solidity" | b"--solidity" | b"-SOLIDITY" | b"--SOLIDITY" => {
                Some(Self::Solidity)
            }
            b"-fs" | b"--fs" | b"-FS" | b"--FS" | b"-fsharp" | b"--fsharp" | b"-FSHARP" | b"--FSHARP" | b"-f#" | b"--f#" | b"-F#" | b"--F#" => {
                Some(Self::FSharp)
            }
            b"-mojo" | b"--mojo" | b"-MOJO" | b"--MOJO" => {
                Some(Self::Mojo)
            }
            b"-v" | b"--v" | b"-V" | b"--V" | b"-verilog" | b"--verilog" | b"-VERILOG" | b"--VERILOG" | b"-sv" | b"--sv" | b"-SV" | b"--SV" => {
                Some(Self::Verilog)
            }
            b"-cu" | b"--cu" | b"-CU" | b"--CU" | b"-cuda" | b"--cuda" | b"-CUDA" | b"--CUDA" => {
                Some(Self::Cuda)
            }
            b"-wgsl" | b"--wgsl" | b"-WGSL" | b"--WGSL" => {
                Some(Self::Wgsl)
            }
            b"-ada" | b"--ada" | b"-ADA" | b"--ADA" => {
                Some(Self::Ada)
            }
            b"-groovy" | b"--groovy" | b"-GROOVY" | b"--GROOVY" | b"-gvy" | b"--gvy" | b"-GVY" | b"--GVY" => {
                Some(Self::Groovy)
            }
            b"-clj" | b"--clj" | b"-CLJ" | b"--CLJ" | b"-clojure" | b"--clojure" | b"-CLOJURE" | b"--CLOJURE" => {
                Some(Self::Clojure)
            }
            b"-asm" | b"--asm" | b"-ASM" | b"--ASM" | b"-s" | b"--s" | b"-assembly" | b"--assembly" => {
                Some(Self::Assembly)
            }
            b"-tcl" | b"--tcl" | b"-TCL" | b"--TCL" => {
                Some(Self::Tcl)
            }
            b"-scm" | b"--scm" | b"-SCM" | b"--SCM" | b"-scheme" | b"--scheme" | b"-rkt" | b"--rkt" => {
                Some(Self::Scheme)
            }
            b"-json" | b"--json-lang" => {
                Some(Self::Json)
            }
            b"-toml" | b"--toml" | b"-TOML" | b"--TOML" => {
                Some(Self::Toml)
            }
            b"-yaml" | b"--yaml" | b"-YAML" | b"--YAML" | b"-yml" | b"--yml" | b"-YML" | b"--YML" => {
                Some(Self::Yaml)
            }
            b"-docker" | b"--docker" | b"-DOCKER" | b"--DOCKER" | b"-dockerfile" | b"--dockerfile" | b"-DOCKERFILE" | b"--DOCKERFILE" => {
                Some(Self::Dockerfile)
            }
            b"-make" | b"--make" | b"-MAKE" | b"--MAKE" | b"-makefile" | b"--makefile" | b"-MAKEFILE" | b"--MAKEFILE" => {
                Some(Self::Makefile)
            }
            b"-cmake" | b"--cmake" | b"-CMAKE" | b"--CMAKE" => {
                Some(Self::CMake)
            }
            b"-glsl" | b"--glsl" | b"-GLSL" | b"--GLSL" => {
                Some(Self::Glsl)
            }
            b"-hlsl" | b"--hlsl" | b"-HLSL" | b"--HLSL" => {
                Some(Self::Hlsl)
            }
            b"-objc" | b"--objc" | b"-OBJC" | b"--OBJC" | b"-objective-c" | b"--objective-c" | b"-OBJECTIVE-C" | b"--OBJECTIVE-C" => {
                Some(Self::ObjectiveC)
            }
            b"-vlang" | b"--vlang" | b"-VLANG" | b"--VLANG" => {
                Some(Self::V)
            }
            b"-lisp" | b"--lisp" | b"-clisp" | b"--clisp" | b"-LISP" | b"--LISP" | b"-CLISP" | b"--CLISP" | b"-common-lisp" | b"--common-lisp" => {
                Some(Self::CommonLisp)
            }
            b"-pas" | b"--pas" | b"-PAS" | b"--PAS" | b"-pascal" | b"--pascal" | b"-PASCAL" | b"--PASCAL" => {
                Some(Self::Pascal)
            }
            b"-ps1" | b"--ps1" | b"-PS1" | b"--PS1" | b"-powershell" | b"--powershell" | b"-POWERSHELL" | b"--POWERSHELL" | b"-pwsh" | b"--pwsh" | b"-PWSH" | b"--PWSH" => {
                Some(Self::PowerShell)
            }
            b"-fish" | b"--fish" | b"-FISH" | b"--FISH" => {
                Some(Self::Fish)
            }
            b"-awk" | b"--awk" | b"-AWK" | b"--AWK" => {
                Some(Self::Awk)
            }
            b"-ha" | b"--ha" | b"-HA" | b"--HA" | b"-hare" | b"--hare" | b"-HARE" | b"--HARE" => {
                Some(Self::Hare)
            }
            b"-html" | b"--html" | b"-HTML" | b"--HTML" | b"-htm" | b"--htm" | b"-HTM" | b"--HTM" => {
                Some(Self::Html)
            }
            b"-xml" | b"--xml" | b"-XML" | b"--XML" => {
                Some(Self::Xml)
            }
            b"-css" | b"--css" | b"-CSS" | b"--CSS" | b"-scss" | b"--scss" | b"-SCSS" | b"--SCSS" | b"-sass" | b"--sass" | b"-SASS" | b"--SASS" | b"-less" | b"--less" | b"-LESS" | b"--LESS" => {
                Some(Self::Css)
            }
            b"-graphql" | b"--graphql" | b"-GRAPHQL" | b"--GRAPHQL" | b"-gql" | b"--gql" | b"-GQL" | b"--GQL" => {
                Some(Self::Graphql)
            }
            b"-proto" | b"--proto" | b"-PROTO" | b"--PROTO" | b"-protobuf" | b"--protobuf" | b"-PROTOBUF" | b"--PROTOBUF" => {
                Some(Self::Protobuf)
            }
            b"-cobol" | b"--cobol" | b"-COBOL" | b"--COBOL" | b"-cbl" | b"--cbl" | b"-CBL" | b"--CBL" => {
                Some(Self::Cobol)
            }
            b"-vhdl" | b"--vhdl" | b"-VHDL" | b"--VHDL" | b"-vhd" | b"--vhd" | b"-VHD" | b"--VHD" => {
                Some(Self::Vhdl)
            }
            b"-ch" | b"--ch" | b"-CH" | b"--CH" | b"-c-header" | b"--c-header" | b"-C-HEADER" | b"--C-HEADER" => {
                Some(Self::CHeader)
            }
            b"-cxxh" | b"--cxxh" | b"-CXXH" | b"--CXXH" | b"-cpp-header" | b"--cpp-header" | b"-CPP-HEADER" | b"--CPP-HEADER" => {
                Some(Self::CppHeader)
            }
            _ => None,
        }
    }
}

#[derive(Clone, Copy)]
pub struct ParseResult<const N: usize> {
    pub keyword_counts: [i64; N],
    pub total_keywords: i64,
    pub lines_processed: i64,
    pub code_lines: i64,
    pub comment_lines: i64,
    pub blank_lines: i64,
}

impl<const N: usize> Default for ParseResult<N> {
    fn default() -> Self {
        Self {
            keyword_counts: [0; N],
            total_keywords: 0,
            lines_processed: 0,
            code_lines: 0,
            comment_lines: 0,
            blank_lines: 0,
        }
    }
}

impl<const N: usize> ParseResult<N> {
    #[inline(always)]
    pub fn add(&mut self, other: &Self) {
        for i in 0..N {
            self.keyword_counts[i] += other.keyword_counts[i];
        }
        self.total_keywords += other.total_keywords;
        self.lines_processed += other.lines_processed;
        self.code_lines += other.code_lines;
        self.comment_lines += other.comment_lines;
        self.blank_lines += other.blank_lines;
    }
}

#[derive(Clone, Copy, Default)]
pub struct LangStats {
    pub c: ParseResult<{ c::KEYWORD_COUNT }>,
    pub cpp: ParseResult<{ cpp::KEYWORD_COUNT }>,
    pub rust: ParseResult<{ rust::KEYWORD_COUNT }>,
    pub python: ParseResult<{ python::KEYWORD_COUNT }>,
    pub perl: ParseResult<{ perl::KEYWORD_COUNT }>,
    pub go: ParseResult<{ go::KEYWORD_COUNT }>,
    pub zig: ParseResult<{ zig::KEYWORD_COUNT }>,
    pub odin: ParseResult<{ odin::KEYWORD_COUNT }>,
    pub javascript: ParseResult<{ javascript::KEYWORD_COUNT }>,
    pub typescript: ParseResult<{ typescript::KEYWORD_COUNT }>,
    pub lua: ParseResult<{ lua::KEYWORD_COUNT }>,
    pub java: ParseResult<{ java::KEYWORD_COUNT }>,
    pub csharp: ParseResult<{ csharp::KEYWORD_COUNT }>,
    pub ruby: ParseResult<{ ruby::KEYWORD_COUNT }>,
    pub php: ParseResult<{ php::KEYWORD_COUNT }>,
    pub swift: ParseResult<{ swift::KEYWORD_COUNT }>,
    pub kotlin: ParseResult<{ kotlin::KEYWORD_COUNT }>,
    pub bash: ParseResult<{ bash::KEYWORD_COUNT }>,
    pub sql: ParseResult<{ sql::KEYWORD_COUNT }>,
    pub dart: ParseResult<{ dart::KEYWORD_COUNT }>,
    pub scala: ParseResult<{ scala::KEYWORD_COUNT }>,
    pub haskell: ParseResult<{ haskell::KEYWORD_COUNT }>,
    pub ocaml: ParseResult<{ ocaml::KEYWORD_COUNT }>,
    pub gleam: ParseResult<{ gleam::KEYWORD_COUNT }>,
    pub d: ParseResult<{ d::KEYWORD_COUNT }>,
    pub nim: ParseResult<{ nim::KEYWORD_COUNT }>,
    pub fortran: ParseResult<{ fortran::KEYWORD_COUNT }>,
    pub elixir: ParseResult<{ elixir::KEYWORD_COUNT }>,
    pub erlang: ParseResult<{ erlang::KEYWORD_COUNT }>,
    pub julia: ParseResult<{ julia::KEYWORD_COUNT }>,
    pub r: ParseResult<{ r::KEYWORD_COUNT }>,
    pub matlab: ParseResult<{ matlab::KEYWORD_COUNT }>,
    pub solidity: ParseResult<{ solidity::KEYWORD_COUNT }>,
    pub fsharp: ParseResult<{ fsharp::KEYWORD_COUNT }>,
    pub mojo: ParseResult<{ mojo::KEYWORD_COUNT }>,
    pub verilog: ParseResult<{ verilog::KEYWORD_COUNT }>,
    pub cuda: ParseResult<{ cuda::KEYWORD_COUNT }>,
    pub wgsl: ParseResult<{ wgsl::KEYWORD_COUNT }>,
    pub ada: ParseResult<{ ada::KEYWORD_COUNT }>,
    pub groovy: ParseResult<{ groovy::KEYWORD_COUNT }>,
    pub clojure: ParseResult<{ clojure::KEYWORD_COUNT }>,
    pub assembly: ParseResult<{ assembly::KEYWORD_COUNT }>,
    pub tcl: ParseResult<{ tcl::KEYWORD_COUNT }>,
    pub scheme: ParseResult<{ scheme::KEYWORD_COUNT }>,
    pub json: ParseResult<{ json::KEYWORD_COUNT }>,
    pub toml: ParseResult<{ toml::KEYWORD_COUNT }>,
    pub yaml: ParseResult<{ yaml::KEYWORD_COUNT }>,
    pub dockerfile: ParseResult<{ dockerfile::KEYWORD_COUNT }>,
    pub makefile: ParseResult<{ makefile::KEYWORD_COUNT }>,
    pub cmake: ParseResult<{ cmake::KEYWORD_COUNT }>,
    pub glsl: ParseResult<{ glsl::KEYWORD_COUNT }>,
    pub hlsl: ParseResult<{ hlsl::KEYWORD_COUNT }>,
    pub objc: ParseResult<{ objc::KEYWORD_COUNT }>,
    pub v: ParseResult<{ v::KEYWORD_COUNT }>,
    pub clisp: ParseResult<{ clisp::KEYWORD_COUNT }>,
    pub pascal: ParseResult<{ pascal::KEYWORD_COUNT }>,
    pub powershell: ParseResult<{ powershell::KEYWORD_COUNT }>,
    pub fish: ParseResult<{ fish::KEYWORD_COUNT }>,
    pub awk: ParseResult<{ awk::KEYWORD_COUNT }>,
    pub hare: ParseResult<{ hare::KEYWORD_COUNT }>,
    pub html: ParseResult<{ html::KEYWORD_COUNT }>,
    pub xml: ParseResult<{ xml::KEYWORD_COUNT }>,
    pub css: ParseResult<{ css::KEYWORD_COUNT }>,
    pub graphql: ParseResult<{ graphql::KEYWORD_COUNT }>,
    pub protobuf: ParseResult<{ protobuf::KEYWORD_COUNT }>,
    pub cobol: ParseResult<{ cobol::KEYWORD_COUNT }>,
    pub vhdl: ParseResult<{ vhdl::KEYWORD_COUNT }>,
    pub c_header: ParseResult<{ c::KEYWORD_COUNT }>,
    pub cpp_header: ParseResult<{ cpp::KEYWORD_COUNT }>,
}

impl LangStats {
    #[inline(always)]
    pub fn add(&mut self, other: &Self) {
        self.c.add(&other.c);
        self.cpp.add(&other.cpp);
        self.rust.add(&other.rust);
        self.python.add(&other.python);
        self.perl.add(&other.perl);
        self.go.add(&other.go);
        self.zig.add(&other.zig);
        self.odin.add(&other.odin);
        self.javascript.add(&other.javascript);
        self.typescript.add(&other.typescript);
        self.lua.add(&other.lua);
        self.java.add(&other.java);
        self.csharp.add(&other.csharp);
        self.ruby.add(&other.ruby);
        self.php.add(&other.php);
        self.swift.add(&other.swift);
        self.kotlin.add(&other.kotlin);
        self.bash.add(&other.bash);
        self.sql.add(&other.sql);
        self.dart.add(&other.dart);
        self.scala.add(&other.scala);
        self.haskell.add(&other.haskell);
        self.ocaml.add(&other.ocaml);
        self.gleam.add(&other.gleam);
        self.d.add(&other.d);
        self.nim.add(&other.nim);
        self.fortran.add(&other.fortran);
        self.elixir.add(&other.elixir);
        self.erlang.add(&other.erlang);
        self.julia.add(&other.julia);
        self.r.add(&other.r);
        self.matlab.add(&other.matlab);
        self.solidity.add(&other.solidity);
        self.fsharp.add(&other.fsharp);
        self.mojo.add(&other.mojo);
        self.verilog.add(&other.verilog);
        self.cuda.add(&other.cuda);
        self.wgsl.add(&other.wgsl);
        self.ada.add(&other.ada);
        self.groovy.add(&other.groovy);
        self.clojure.add(&other.clojure);
        self.assembly.add(&other.assembly);
        self.tcl.add(&other.tcl);
        self.scheme.add(&other.scheme);
        self.json.add(&other.json);
        self.toml.add(&other.toml);
        self.yaml.add(&other.yaml);
        self.dockerfile.add(&other.dockerfile);
        self.makefile.add(&other.makefile);
        self.cmake.add(&other.cmake);
        self.glsl.add(&other.glsl);
        self.hlsl.add(&other.hlsl);
        self.objc.add(&other.objc);
        self.v.add(&other.v);
        self.clisp.add(&other.clisp);
        self.pascal.add(&other.pascal);
        self.powershell.add(&other.powershell);
        self.fish.add(&other.fish);
        self.awk.add(&other.awk);
        self.hare.add(&other.hare);
        self.html.add(&other.html);
        self.xml.add(&other.xml);
        self.css.add(&other.css);
        self.graphql.add(&other.graphql);
        self.protobuf.add(&other.protobuf);
        self.cobol.add(&other.cobol);
        self.vhdl.add(&other.vhdl);
        self.c_header.add(&other.c_header);
        self.cpp_header.add(&other.cpp_header);
    }

    #[inline(always)]
    pub fn lines(&self, lang: LangId) -> i64 {
        match lang {
            LangId::C => self.c.lines_processed,
            LangId::Cpp => self.cpp.lines_processed,
            LangId::Rust => self.rust.lines_processed,
            LangId::Python => self.python.lines_processed,
            LangId::Perl => self.perl.lines_processed,
            LangId::Go => self.go.lines_processed,
            LangId::Zig => self.zig.lines_processed,
            LangId::Odin => self.odin.lines_processed,
            LangId::JavaScript => self.javascript.lines_processed,
            LangId::TypeScript => self.typescript.lines_processed,
            LangId::Lua => self.lua.lines_processed,
            LangId::Java => self.java.lines_processed,
            LangId::CSharp => self.csharp.lines_processed,
            LangId::Ruby => self.ruby.lines_processed,
            LangId::Php => self.php.lines_processed,
            LangId::Swift => self.swift.lines_processed,
            LangId::Kotlin => self.kotlin.lines_processed,
            LangId::Bash => self.bash.lines_processed,
            LangId::Sql => self.sql.lines_processed,
            LangId::Dart => self.dart.lines_processed,
            LangId::Scala => self.scala.lines_processed,
            LangId::Haskell => self.haskell.lines_processed,
            LangId::OCaml => self.ocaml.lines_processed,
            LangId::Gleam => self.gleam.lines_processed,
            LangId::D => self.d.lines_processed,
            LangId::Nim => self.nim.lines_processed,
            LangId::Fortran => self.fortran.lines_processed,
            LangId::Elixir => self.elixir.lines_processed,
            LangId::Erlang => self.erlang.lines_processed,
            LangId::Julia => self.julia.lines_processed,
            LangId::R => self.r.lines_processed,
            LangId::Matlab => self.matlab.lines_processed,
            LangId::Solidity => self.solidity.lines_processed,
            LangId::FSharp => self.fsharp.lines_processed,
            LangId::Mojo => self.mojo.lines_processed,
            LangId::Verilog => self.verilog.lines_processed,
            LangId::Cuda => self.cuda.lines_processed,
            LangId::Wgsl => self.wgsl.lines_processed,
            LangId::Ada => self.ada.lines_processed,
            LangId::Groovy => self.groovy.lines_processed,
            LangId::Clojure => self.clojure.lines_processed,
            LangId::Assembly => self.assembly.lines_processed,
            LangId::Tcl => self.tcl.lines_processed,
            LangId::Scheme => self.scheme.lines_processed,
            LangId::Json => self.json.lines_processed,
            LangId::Toml => self.toml.lines_processed,
            LangId::Yaml => self.yaml.lines_processed,
            LangId::Dockerfile => self.dockerfile.lines_processed,
            LangId::Makefile => self.makefile.lines_processed,
            LangId::CMake => self.cmake.lines_processed,
            LangId::Glsl => self.glsl.lines_processed,
            LangId::Hlsl => self.hlsl.lines_processed,
            LangId::ObjectiveC => self.objc.lines_processed,
            LangId::V => self.v.lines_processed,
            LangId::CommonLisp => self.clisp.lines_processed,
            LangId::Pascal => self.pascal.lines_processed,
            LangId::PowerShell => self.powershell.lines_processed,
            LangId::Fish => self.fish.lines_processed,
            LangId::Awk => self.awk.lines_processed,
            LangId::Hare => self.hare.lines_processed,
            LangId::Html => self.html.lines_processed,
            LangId::Xml => self.xml.lines_processed,
            LangId::Css => self.css.lines_processed,
            LangId::Graphql => self.graphql.lines_processed,
            LangId::Protobuf => self.protobuf.lines_processed,
            LangId::Cobol => self.cobol.lines_processed,
            LangId::Vhdl => self.vhdl.lines_processed,
            LangId::CHeader => self.c_header.lines_processed,
            LangId::CppHeader => self.cpp_header.lines_processed,
        }
    }

    #[inline(always)]
    pub fn code_lines(&self, lang: LangId) -> i64 {
        match lang {
            LangId::C => self.c.code_lines,
            LangId::Cpp => self.cpp.code_lines,
            LangId::Rust => self.rust.code_lines,
            LangId::Python => self.python.code_lines,
            LangId::Perl => self.perl.code_lines,
            LangId::Go => self.go.code_lines,
            LangId::Zig => self.zig.code_lines,
            LangId::Odin => self.odin.code_lines,
            LangId::JavaScript => self.javascript.code_lines,
            LangId::TypeScript => self.typescript.code_lines,
            LangId::Lua => self.lua.code_lines,
            LangId::Java => self.java.code_lines,
            LangId::CSharp => self.csharp.code_lines,
            LangId::Ruby => self.ruby.code_lines,
            LangId::Php => self.php.code_lines,
            LangId::Swift => self.swift.code_lines,
            LangId::Kotlin => self.kotlin.code_lines,
            LangId::Bash => self.bash.code_lines,
            LangId::Sql => self.sql.code_lines,
            LangId::Dart => self.dart.code_lines,
            LangId::Scala => self.scala.code_lines,
            LangId::Haskell => self.haskell.code_lines,
            LangId::OCaml => self.ocaml.code_lines,
            LangId::Gleam => self.gleam.code_lines,
            LangId::D => self.d.code_lines,
            LangId::Nim => self.nim.code_lines,
            LangId::Fortran => self.fortran.code_lines,
            LangId::Elixir => self.elixir.code_lines,
            LangId::Erlang => self.erlang.code_lines,
            LangId::Julia => self.julia.code_lines,
            LangId::R => self.r.code_lines,
            LangId::Matlab => self.matlab.code_lines,
            LangId::Solidity => self.solidity.code_lines,
            LangId::FSharp => self.fsharp.code_lines,
            LangId::Mojo => self.mojo.code_lines,
            LangId::Verilog => self.verilog.code_lines,
            LangId::Cuda => self.cuda.code_lines,
            LangId::Wgsl => self.wgsl.code_lines,
            LangId::Ada => self.ada.code_lines,
            LangId::Groovy => self.groovy.code_lines,
            LangId::Clojure => self.clojure.code_lines,
            LangId::Assembly => self.assembly.code_lines,
            LangId::Tcl => self.tcl.code_lines,
            LangId::Scheme => self.scheme.code_lines,
            LangId::Json => self.json.code_lines,
            LangId::Toml => self.toml.code_lines,
            LangId::Yaml => self.yaml.code_lines,
            LangId::Dockerfile => self.dockerfile.code_lines,
            LangId::Makefile => self.makefile.code_lines,
            LangId::CMake => self.cmake.code_lines,
            LangId::Glsl => self.glsl.code_lines,
            LangId::Hlsl => self.hlsl.code_lines,
            LangId::ObjectiveC => self.objc.code_lines,
            LangId::V => self.v.code_lines,
            LangId::CommonLisp => self.clisp.code_lines,
            LangId::Pascal => self.pascal.code_lines,
            LangId::PowerShell => self.powershell.code_lines,
            LangId::Fish => self.fish.code_lines,
            LangId::Awk => self.awk.code_lines,
            LangId::Hare => self.hare.code_lines,
            LangId::Html => self.html.code_lines,
            LangId::Xml => self.xml.code_lines,
            LangId::Css => self.css.code_lines,
            LangId::Graphql => self.graphql.code_lines,
            LangId::Protobuf => self.protobuf.code_lines,
            LangId::Cobol => self.cobol.code_lines,
            LangId::Vhdl => self.vhdl.code_lines,
            LangId::CHeader => self.c_header.code_lines,
            LangId::CppHeader => self.cpp_header.code_lines,
        }
    }

    #[inline(always)]
    pub fn comment_lines(&self, lang: LangId) -> i64 {
        match lang {
            LangId::C => self.c.comment_lines,
            LangId::Cpp => self.cpp.comment_lines,
            LangId::Rust => self.rust.comment_lines,
            LangId::Python => self.python.comment_lines,
            LangId::Perl => self.perl.comment_lines,
            LangId::Go => self.go.comment_lines,
            LangId::Zig => self.zig.comment_lines,
            LangId::Odin => self.odin.comment_lines,
            LangId::JavaScript => self.javascript.comment_lines,
            LangId::TypeScript => self.typescript.comment_lines,
            LangId::Lua => self.lua.comment_lines,
            LangId::Java => self.java.comment_lines,
            LangId::CSharp => self.csharp.comment_lines,
            LangId::Ruby => self.ruby.comment_lines,
            LangId::Php => self.php.comment_lines,
            LangId::Swift => self.swift.comment_lines,
            LangId::Kotlin => self.kotlin.comment_lines,
            LangId::Bash => self.bash.comment_lines,
            LangId::Sql => self.sql.comment_lines,
            LangId::Dart => self.dart.comment_lines,
            LangId::Scala => self.scala.comment_lines,
            LangId::Haskell => self.haskell.comment_lines,
            LangId::OCaml => self.ocaml.comment_lines,
            LangId::Gleam => self.gleam.comment_lines,
            LangId::D => self.d.comment_lines,
            LangId::Nim => self.nim.comment_lines,
            LangId::Fortran => self.fortran.comment_lines,
            LangId::Elixir => self.elixir.comment_lines,
            LangId::Erlang => self.erlang.comment_lines,
            LangId::Julia => self.julia.comment_lines,
            LangId::R => self.r.comment_lines,
            LangId::Matlab => self.matlab.comment_lines,
            LangId::Solidity => self.solidity.comment_lines,
            LangId::FSharp => self.fsharp.comment_lines,
            LangId::Mojo => self.mojo.comment_lines,
            LangId::Verilog => self.verilog.comment_lines,
            LangId::Cuda => self.cuda.comment_lines,
            LangId::Wgsl => self.wgsl.comment_lines,
            LangId::Ada => self.ada.comment_lines,
            LangId::Groovy => self.groovy.comment_lines,
            LangId::Clojure => self.clojure.comment_lines,
            LangId::Assembly => self.assembly.comment_lines,
            LangId::Tcl => self.tcl.comment_lines,
            LangId::Scheme => self.scheme.comment_lines,
            LangId::Json => self.json.comment_lines,
            LangId::Toml => self.toml.comment_lines,
            LangId::Yaml => self.yaml.comment_lines,
            LangId::Dockerfile => self.dockerfile.comment_lines,
            LangId::Makefile => self.makefile.comment_lines,
            LangId::CMake => self.cmake.comment_lines,
            LangId::Glsl => self.glsl.comment_lines,
            LangId::Hlsl => self.hlsl.comment_lines,
            LangId::ObjectiveC => self.objc.comment_lines,
            LangId::V => self.v.comment_lines,
            LangId::CommonLisp => self.clisp.comment_lines,
            LangId::Pascal => self.pascal.comment_lines,
            LangId::PowerShell => self.powershell.comment_lines,
            LangId::Fish => self.fish.comment_lines,
            LangId::Awk => self.awk.comment_lines,
            LangId::Hare => self.hare.comment_lines,
            LangId::Html => self.html.comment_lines,
            LangId::Xml => self.xml.comment_lines,
            LangId::Css => self.css.comment_lines,
            LangId::Graphql => self.graphql.comment_lines,
            LangId::Protobuf => self.protobuf.comment_lines,
            LangId::Cobol => self.cobol.comment_lines,
            LangId::Vhdl => self.vhdl.comment_lines,
            LangId::CHeader => self.c_header.comment_lines,
            LangId::CppHeader => self.cpp_header.comment_lines,
        }
    }

    #[inline(always)]
    pub fn blank_lines(&self, lang: LangId) -> i64 {
        match lang {
            LangId::C => self.c.blank_lines,
            LangId::Cpp => self.cpp.blank_lines,
            LangId::Rust => self.rust.blank_lines,
            LangId::Python => self.python.blank_lines,
            LangId::Perl => self.perl.blank_lines,
            LangId::Go => self.go.blank_lines,
            LangId::Zig => self.zig.blank_lines,
            LangId::Odin => self.odin.blank_lines,
            LangId::JavaScript => self.javascript.blank_lines,
            LangId::TypeScript => self.typescript.blank_lines,
            LangId::Lua => self.lua.blank_lines,
            LangId::Java => self.java.blank_lines,
            LangId::CSharp => self.csharp.blank_lines,
            LangId::Ruby => self.ruby.blank_lines,
            LangId::Php => self.php.blank_lines,
            LangId::Swift => self.swift.blank_lines,
            LangId::Kotlin => self.kotlin.blank_lines,
            LangId::Bash => self.bash.blank_lines,
            LangId::Sql => self.sql.blank_lines,
            LangId::Dart => self.dart.blank_lines,
            LangId::Scala => self.scala.blank_lines,
            LangId::Haskell => self.haskell.blank_lines,
            LangId::OCaml => self.ocaml.blank_lines,
            LangId::Gleam => self.gleam.blank_lines,
            LangId::D => self.d.blank_lines,
            LangId::Nim => self.nim.blank_lines,
            LangId::Fortran => self.fortran.blank_lines,
            LangId::Elixir => self.elixir.blank_lines,
            LangId::Erlang => self.erlang.blank_lines,
            LangId::Julia => self.julia.blank_lines,
            LangId::R => self.r.blank_lines,
            LangId::Matlab => self.matlab.blank_lines,
            LangId::Solidity => self.solidity.blank_lines,
            LangId::FSharp => self.fsharp.blank_lines,
            LangId::Mojo => self.mojo.blank_lines,
            LangId::Verilog => self.verilog.blank_lines,
            LangId::Cuda => self.cuda.blank_lines,
            LangId::Wgsl => self.wgsl.blank_lines,
            LangId::Ada => self.ada.blank_lines,
            LangId::Groovy => self.groovy.blank_lines,
            LangId::Clojure => self.clojure.blank_lines,
            LangId::Assembly => self.assembly.blank_lines,
            LangId::Tcl => self.tcl.blank_lines,
            LangId::Scheme => self.scheme.blank_lines,
            LangId::Json => self.json.blank_lines,
            LangId::Toml => self.toml.blank_lines,
            LangId::Yaml => self.yaml.blank_lines,
            LangId::Dockerfile => self.dockerfile.blank_lines,
            LangId::Makefile => self.makefile.blank_lines,
            LangId::CMake => self.cmake.blank_lines,
            LangId::Glsl => self.glsl.blank_lines,
            LangId::Hlsl => self.hlsl.blank_lines,
            LangId::ObjectiveC => self.objc.blank_lines,
            LangId::V => self.v.blank_lines,
            LangId::CommonLisp => self.clisp.blank_lines,
            LangId::Pascal => self.pascal.blank_lines,
            LangId::PowerShell => self.powershell.blank_lines,
            LangId::Fish => self.fish.blank_lines,
            LangId::Awk => self.awk.blank_lines,
            LangId::Hare => self.hare.blank_lines,
            LangId::Html => self.html.blank_lines,
            LangId::Xml => self.xml.blank_lines,
            LangId::Css => self.css.blank_lines,
            LangId::Graphql => self.graphql.blank_lines,
            LangId::Protobuf => self.protobuf.blank_lines,
            LangId::Cobol => self.cobol.blank_lines,
            LangId::Vhdl => self.vhdl.blank_lines,
            LangId::CHeader => self.c_header.blank_lines,
            LangId::CppHeader => self.cpp_header.blank_lines,
        }
    }

    #[inline(always)]
    pub fn total_keywords(&self, lang: LangId) -> i64 {
        match lang {
            LangId::C => self.c.total_keywords,
            LangId::Cpp => self.cpp.total_keywords,
            LangId::Rust => self.rust.total_keywords,
            LangId::Python => self.python.total_keywords,
            LangId::Perl => self.perl.total_keywords,
            LangId::Go => self.go.total_keywords,
            LangId::Zig => self.zig.total_keywords,
            LangId::Odin => self.odin.total_keywords,
            LangId::JavaScript => self.javascript.total_keywords,
            LangId::TypeScript => self.typescript.total_keywords,
            LangId::Lua => self.lua.total_keywords,
            LangId::Java => self.java.total_keywords,
            LangId::CSharp => self.csharp.total_keywords,
            LangId::Ruby => self.ruby.total_keywords,
            LangId::Php => self.php.total_keywords,
            LangId::Swift => self.swift.total_keywords,
            LangId::Kotlin => self.kotlin.total_keywords,
            LangId::Bash => self.bash.total_keywords,
            LangId::Sql => self.sql.total_keywords,
            LangId::Dart => self.dart.total_keywords,
            LangId::Scala => self.scala.total_keywords,
            LangId::Haskell => self.haskell.total_keywords,
            LangId::OCaml => self.ocaml.total_keywords,
            LangId::Gleam => self.gleam.total_keywords,
            LangId::D => self.d.total_keywords,
            LangId::Nim => self.nim.total_keywords,
            LangId::Fortran => self.fortran.total_keywords,
            LangId::Elixir => self.elixir.total_keywords,
            LangId::Erlang => self.erlang.total_keywords,
            LangId::Julia => self.julia.total_keywords,
            LangId::R => self.r.total_keywords,
            LangId::Matlab => self.matlab.total_keywords,
            LangId::Solidity => self.solidity.total_keywords,
            LangId::FSharp => self.fsharp.total_keywords,
            LangId::Mojo => self.mojo.total_keywords,
            LangId::Verilog => self.verilog.total_keywords,
            LangId::Cuda => self.cuda.total_keywords,
            LangId::Wgsl => self.wgsl.total_keywords,
            LangId::Ada => self.ada.total_keywords,
            LangId::Groovy => self.groovy.total_keywords,
            LangId::Clojure => self.clojure.total_keywords,
            LangId::Assembly => self.assembly.total_keywords,
            LangId::Tcl => self.tcl.total_keywords,
            LangId::Scheme => self.scheme.total_keywords,
            LangId::Json => self.json.total_keywords,
            LangId::Toml => self.toml.total_keywords,
            LangId::Yaml => self.yaml.total_keywords,
            LangId::Dockerfile => self.dockerfile.total_keywords,
            LangId::Makefile => self.makefile.total_keywords,
            LangId::CMake => self.cmake.total_keywords,
            LangId::Glsl => self.glsl.total_keywords,
            LangId::Hlsl => self.hlsl.total_keywords,
            LangId::ObjectiveC => self.objc.total_keywords,
            LangId::V => self.v.total_keywords,
            LangId::CommonLisp => self.clisp.total_keywords,
            LangId::Pascal => self.pascal.total_keywords,
            LangId::PowerShell => self.powershell.total_keywords,
            LangId::Fish => self.fish.total_keywords,
            LangId::Awk => self.awk.total_keywords,
            LangId::Hare => self.hare.total_keywords,
            LangId::Html => self.html.total_keywords,
            LangId::Xml => self.xml.total_keywords,
            LangId::Css => self.css.total_keywords,
            LangId::Graphql => self.graphql.total_keywords,
            LangId::Protobuf => self.protobuf.total_keywords,
            LangId::Cobol => self.cobol.total_keywords,
            LangId::Vhdl => self.vhdl.total_keywords,
            LangId::CHeader => self.c_header.total_keywords,
            LangId::CppHeader => self.cpp_header.total_keywords,
        }
    }

    #[inline(always)]
    pub fn keyword_names(lang: LangId) -> &'static [&'static str] {
        match lang {
            LangId::C => &c::KEYWORD_NAMES,
            LangId::Cpp => &cpp::KEYWORD_NAMES,
            LangId::Rust => &rust::KEYWORD_NAMES,
            LangId::Python => &python::KEYWORD_NAMES,
            LangId::Perl => &perl::KEYWORD_NAMES,
            LangId::Go => &go::KEYWORD_NAMES,
            LangId::Zig => &zig::KEYWORD_NAMES,
            LangId::Odin => &odin::KEYWORD_NAMES,
            LangId::JavaScript => &javascript::KEYWORD_NAMES,
            LangId::TypeScript => &typescript::KEYWORD_NAMES,
            LangId::Lua => &lua::KEYWORD_NAMES,
            LangId::Java => &java::KEYWORD_NAMES,
            LangId::CSharp => &csharp::KEYWORD_NAMES,
            LangId::Ruby => &ruby::KEYWORD_NAMES,
            LangId::Php => &php::KEYWORD_NAMES,
            LangId::Swift => &swift::KEYWORD_NAMES,
            LangId::Kotlin => &kotlin::KEYWORD_NAMES,
            LangId::Bash => &bash::KEYWORD_NAMES,
            LangId::Sql => &sql::KEYWORD_NAMES,
            LangId::Dart => &dart::KEYWORD_NAMES,
            LangId::Scala => &scala::KEYWORD_NAMES,
            LangId::Haskell => &haskell::KEYWORD_NAMES,
            LangId::OCaml => &ocaml::KEYWORD_NAMES,
            LangId::Gleam => &gleam::KEYWORD_NAMES,
            LangId::D => &d::KEYWORD_NAMES,
            LangId::Nim => &nim::KEYWORD_NAMES,
            LangId::Fortran => &fortran::KEYWORD_NAMES,
            LangId::Elixir => &elixir::KEYWORD_NAMES,
            LangId::Erlang => &erlang::KEYWORD_NAMES,
            LangId::Julia => &julia::KEYWORD_NAMES,
            LangId::R => &r::KEYWORD_NAMES,
            LangId::Matlab => &matlab::KEYWORD_NAMES,
            LangId::Solidity => &solidity::KEYWORD_NAMES,
            LangId::FSharp => &fsharp::KEYWORD_NAMES,
            LangId::Mojo => &mojo::KEYWORD_NAMES,
            LangId::Verilog => &verilog::KEYWORD_NAMES,
            LangId::Cuda => &cuda::KEYWORD_NAMES,
            LangId::Wgsl => &wgsl::KEYWORD_NAMES,
            LangId::Ada => &ada::KEYWORD_NAMES,
            LangId::Groovy => &groovy::KEYWORD_NAMES,
            LangId::Clojure => &clojure::KEYWORD_NAMES,
            LangId::Assembly => &assembly::KEYWORD_NAMES,
            LangId::Tcl => &tcl::KEYWORD_NAMES,
            LangId::Scheme => &scheme::KEYWORD_NAMES,
            LangId::Json => &json::KEYWORD_NAMES,
            LangId::Toml => &toml::KEYWORD_NAMES,
            LangId::Yaml => &yaml::KEYWORD_NAMES,
            LangId::Dockerfile => &dockerfile::KEYWORD_NAMES,
            LangId::Makefile => &makefile::KEYWORD_NAMES,
            LangId::CMake => &cmake::KEYWORD_NAMES,
            LangId::Glsl => &glsl::KEYWORD_NAMES,
            LangId::Hlsl => &hlsl::KEYWORD_NAMES,
            LangId::ObjectiveC => &objc::KEYWORD_NAMES,
            LangId::V => &v::KEYWORD_NAMES,
            LangId::CommonLisp => &clisp::KEYWORD_NAMES,
            LangId::Pascal => &pascal::KEYWORD_NAMES,
            LangId::PowerShell => &powershell::KEYWORD_NAMES,
            LangId::Fish => &fish::KEYWORD_NAMES,
            LangId::Awk => &awk::KEYWORD_NAMES,
            LangId::Hare => &hare::KEYWORD_NAMES,
            LangId::Html => &html::KEYWORD_NAMES,
            LangId::Xml => &xml::KEYWORD_NAMES,
            LangId::Css => &css::KEYWORD_NAMES,
            LangId::Graphql => &graphql::KEYWORD_NAMES,
            LangId::Protobuf => &protobuf::KEYWORD_NAMES,
            LangId::Cobol => &cobol::KEYWORD_NAMES,
            LangId::Vhdl => &vhdl::KEYWORD_NAMES,
            LangId::CHeader => &c::KEYWORD_NAMES,
            LangId::CppHeader => &cpp::KEYWORD_NAMES,
        }
    }

    #[inline(always)]
    pub fn keyword_counts(&self, lang: LangId) -> &[i64] {
        match lang {
            LangId::C => &self.c.keyword_counts,
            LangId::Cpp => &self.cpp.keyword_counts,
            LangId::Rust => &self.rust.keyword_counts,
            LangId::Python => &self.python.keyword_counts,
            LangId::Perl => &self.perl.keyword_counts,
            LangId::Go => &self.go.keyword_counts,
            LangId::Zig => &self.zig.keyword_counts,
            LangId::Odin => &self.odin.keyword_counts,
            LangId::JavaScript => &self.javascript.keyword_counts,
            LangId::TypeScript => &self.typescript.keyword_counts,
            LangId::Lua => &self.lua.keyword_counts,
            LangId::Java => &self.java.keyword_counts,
            LangId::CSharp => &self.csharp.keyword_counts,
            LangId::Ruby => &self.ruby.keyword_counts,
            LangId::Php => &self.php.keyword_counts,
            LangId::Swift => &self.swift.keyword_counts,
            LangId::Kotlin => &self.kotlin.keyword_counts,
            LangId::Bash => &self.bash.keyword_counts,
            LangId::Sql => &self.sql.keyword_counts,
            LangId::Dart => &self.dart.keyword_counts,
            LangId::Scala => &self.scala.keyword_counts,
            LangId::Haskell => &self.haskell.keyword_counts,
            LangId::OCaml => &self.ocaml.keyword_counts,
            LangId::Gleam => &self.gleam.keyword_counts,
            LangId::D => &self.d.keyword_counts,
            LangId::Nim => &self.nim.keyword_counts,
            LangId::Fortran => &self.fortran.keyword_counts,
            LangId::Elixir => &self.elixir.keyword_counts,
            LangId::Erlang => &self.erlang.keyword_counts,
            LangId::Julia => &self.julia.keyword_counts,
            LangId::R => &self.r.keyword_counts,
            LangId::Matlab => &self.matlab.keyword_counts,
            LangId::Solidity => &self.solidity.keyword_counts,
            LangId::FSharp => &self.fsharp.keyword_counts,
            LangId::Mojo => &self.mojo.keyword_counts,
            LangId::Verilog => &self.verilog.keyword_counts,
            LangId::Cuda => &self.cuda.keyword_counts,
            LangId::Wgsl => &self.wgsl.keyword_counts,
            LangId::Ada => &self.ada.keyword_counts,
            LangId::Groovy => &self.groovy.keyword_counts,
            LangId::Clojure => &self.clojure.keyword_counts,
            LangId::Assembly => &self.assembly.keyword_counts,
            LangId::Tcl => &self.tcl.keyword_counts,
            LangId::Scheme => &self.scheme.keyword_counts,
            LangId::Json => &self.json.keyword_counts,
            LangId::Toml => &self.toml.keyword_counts,
            LangId::Yaml => &self.yaml.keyword_counts,
            LangId::Dockerfile => &self.dockerfile.keyword_counts,
            LangId::Makefile => &self.makefile.keyword_counts,
            LangId::CMake => &self.cmake.keyword_counts,
            LangId::Glsl => &self.glsl.keyword_counts,
            LangId::Hlsl => &self.hlsl.keyword_counts,
            LangId::ObjectiveC => &self.objc.keyword_counts,
            LangId::V => &self.v.keyword_counts,
            LangId::CommonLisp => &self.clisp.keyword_counts,
            LangId::Pascal => &self.pascal.keyword_counts,
            LangId::PowerShell => &self.powershell.keyword_counts,
            LangId::Fish => &self.fish.keyword_counts,
            LangId::Awk => &self.awk.keyword_counts,
            LangId::Hare => &self.hare.keyword_counts,
            LangId::Html => &self.html.keyword_counts,
            LangId::Xml => &self.xml.keyword_counts,
            LangId::Css => &self.css.keyword_counts,
            LangId::Graphql => &self.graphql.keyword_counts,
            LangId::Protobuf => &self.protobuf.keyword_counts,
            LangId::Cobol => &self.cobol.keyword_counts,
            LangId::Vhdl => &self.vhdl.keyword_counts,
            LangId::CHeader => &self.c_header.keyword_counts,
            LangId::CppHeader => &self.cpp_header.keyword_counts,
        }
    }

    #[cfg(test)]
    pub fn keyword_lens(lang: LangId) -> &'static [u8] {
        match lang {
            LangId::C => &c::KEYWORD_LENS,
            LangId::Cpp => &cpp::KEYWORD_LENS,
            LangId::Rust => &rust::KEYWORD_LENS,
            LangId::Python => &python::KEYWORD_LENS,
            LangId::Perl => &perl::KEYWORD_LENS,
            LangId::Go => &go::KEYWORD_LENS,
            LangId::Zig => &zig::KEYWORD_LENS,
            LangId::Odin => &odin::KEYWORD_LENS,
            LangId::JavaScript => &javascript::KEYWORD_LENS,
            LangId::TypeScript => &typescript::KEYWORD_LENS,
            LangId::Lua => &lua::KEYWORD_LENS,
            LangId::Java => &java::KEYWORD_LENS,
            LangId::CSharp => &csharp::KEYWORD_LENS,
            LangId::Ruby => &ruby::KEYWORD_LENS,
            LangId::Php => &php::KEYWORD_LENS,
            LangId::Swift => &swift::KEYWORD_LENS,
            LangId::Kotlin => &kotlin::KEYWORD_LENS,
            LangId::Bash => &bash::KEYWORD_LENS,
            LangId::Sql => &sql::KEYWORD_LENS,
            LangId::Dart => &dart::KEYWORD_LENS,
            LangId::Scala => &scala::KEYWORD_LENS,
            LangId::Haskell => &haskell::KEYWORD_LENS,
            LangId::OCaml => &ocaml::KEYWORD_LENS,
            LangId::Gleam => &gleam::KEYWORD_LENS,
            LangId::D => &d::KEYWORD_LENS,
            LangId::Nim => &nim::KEYWORD_LENS,
            LangId::Fortran => &fortran::KEYWORD_LENS,
            LangId::Elixir => &elixir::KEYWORD_LENS,
            LangId::Erlang => &erlang::KEYWORD_LENS,
            LangId::Julia => &julia::KEYWORD_LENS,
            LangId::R => &r::KEYWORD_LENS,
            LangId::Matlab => &matlab::KEYWORD_LENS,
            LangId::Solidity => &solidity::KEYWORD_LENS,
            LangId::FSharp => &fsharp::KEYWORD_LENS,
            LangId::Mojo => &mojo::KEYWORD_LENS,
            LangId::Verilog => &verilog::KEYWORD_LENS,
            LangId::Cuda => &cuda::KEYWORD_LENS,
            LangId::Wgsl => &wgsl::KEYWORD_LENS,
            LangId::Ada => &ada::KEYWORD_LENS,
            LangId::Groovy => &groovy::KEYWORD_LENS,
            LangId::Clojure => &clojure::KEYWORD_LENS,
            LangId::Assembly => &assembly::KEYWORD_LENS,
            LangId::Tcl => &tcl::KEYWORD_LENS,
            LangId::Scheme => &scheme::KEYWORD_LENS,
            LangId::Json => &json::KEYWORD_LENS,
            LangId::Toml => &toml::KEYWORD_LENS,
            LangId::Yaml => &yaml::KEYWORD_LENS,
            LangId::Dockerfile => &dockerfile::KEYWORD_LENS,
            LangId::Makefile => &makefile::KEYWORD_LENS,
            LangId::CMake => &cmake::KEYWORD_LENS,
            LangId::Glsl => &glsl::KEYWORD_LENS,
            LangId::Hlsl => &hlsl::KEYWORD_LENS,
            LangId::ObjectiveC => &objc::KEYWORD_LENS,
            LangId::V => &v::KEYWORD_LENS,
            LangId::CommonLisp => &clisp::KEYWORD_LENS,
            LangId::Pascal => &pascal::KEYWORD_LENS,
            LangId::PowerShell => &powershell::KEYWORD_LENS,
            LangId::Fish => &fish::KEYWORD_LENS,
            LangId::Awk => &awk::KEYWORD_LENS,
            LangId::Hare => &hare::KEYWORD_LENS,
            LangId::Html => &html::KEYWORD_LENS,
            LangId::Xml => &xml::KEYWORD_LENS,
            LangId::Css => &css::KEYWORD_LENS,
            LangId::Graphql => &graphql::KEYWORD_LENS,
            LangId::Protobuf => &protobuf::KEYWORD_LENS,
            LangId::Cobol => &cobol::KEYWORD_LENS,
            LangId::Vhdl => &vhdl::KEYWORD_LENS,
            LangId::CHeader => &c::KEYWORD_LENS,
            LangId::CppHeader => &cpp::KEYWORD_LENS,
        }
    }

    #[cfg(test)]
    pub fn kw_min_max(lang: LangId) -> (usize, usize) {
        match lang {
            LangId::C => (c::KW_MIN_LEN, c::KW_MAX_LEN),
            LangId::Cpp => (cpp::KW_MIN_LEN, cpp::KW_MAX_LEN),
            LangId::Rust => (rust::KW_MIN_LEN, rust::KW_MAX_LEN),
            LangId::Python => (python::KW_MIN_LEN, python::KW_MAX_LEN),
            LangId::Perl => (perl::KW_MIN_LEN, perl::KW_MAX_LEN),
            LangId::Go => (go::KW_MIN_LEN, go::KW_MAX_LEN),
            LangId::Zig => (zig::KW_MIN_LEN, zig::KW_MAX_LEN),
            LangId::Odin => (odin::KW_MIN_LEN, odin::KW_MAX_LEN),
            LangId::JavaScript => (javascript::KW_MIN_LEN, javascript::KW_MAX_LEN),
            LangId::TypeScript => (typescript::KW_MIN_LEN, typescript::KW_MAX_LEN),
            LangId::Lua => (lua::KW_MIN_LEN, lua::KW_MAX_LEN),
            LangId::Java => (java::KW_MIN_LEN, java::KW_MAX_LEN),
            LangId::CSharp => (csharp::KW_MIN_LEN, csharp::KW_MAX_LEN),
            LangId::Ruby => (ruby::KW_MIN_LEN, ruby::KW_MAX_LEN),
            LangId::Php => (php::KW_MIN_LEN, php::KW_MAX_LEN),
            LangId::Swift => (swift::KW_MIN_LEN, swift::KW_MAX_LEN),
            LangId::Kotlin => (kotlin::KW_MIN_LEN, kotlin::KW_MAX_LEN),
            LangId::Bash => (bash::KW_MIN_LEN, bash::KW_MAX_LEN),
            LangId::Sql => (sql::KW_MIN_LEN, sql::KW_MAX_LEN),
            LangId::Dart => (dart::KW_MIN_LEN, dart::KW_MAX_LEN),
            LangId::Scala => (scala::KW_MIN_LEN, scala::KW_MAX_LEN),
            LangId::Haskell => (haskell::KW_MIN_LEN, haskell::KW_MAX_LEN),
            LangId::OCaml => (ocaml::KW_MIN_LEN, ocaml::KW_MAX_LEN),
            LangId::Gleam => (gleam::KW_MIN_LEN, gleam::KW_MAX_LEN),
            LangId::D => (d::KW_MIN_LEN, d::KW_MAX_LEN),
            LangId::Nim => (nim::KW_MIN_LEN, nim::KW_MAX_LEN),
            LangId::Fortran => (fortran::KW_MIN_LEN, fortran::KW_MAX_LEN),
            LangId::Elixir => (elixir::KW_MIN_LEN, elixir::KW_MAX_LEN),
            LangId::Erlang => (erlang::KW_MIN_LEN, erlang::KW_MAX_LEN),
            LangId::Julia => (julia::KW_MIN_LEN, julia::KW_MAX_LEN),
            LangId::R => (r::KW_MIN_LEN, r::KW_MAX_LEN),
            LangId::Matlab => (matlab::KW_MIN_LEN, matlab::KW_MAX_LEN),
            LangId::Solidity => (solidity::KW_MIN_LEN, solidity::KW_MAX_LEN),
            LangId::FSharp => (fsharp::KW_MIN_LEN, fsharp::KW_MAX_LEN),
            LangId::Mojo => (mojo::KW_MIN_LEN, mojo::KW_MAX_LEN),
            LangId::Verilog => (verilog::KW_MIN_LEN, verilog::KW_MAX_LEN),
            LangId::Cuda => (cuda::KW_MIN_LEN, cuda::KW_MAX_LEN),
            LangId::Wgsl => (wgsl::KW_MIN_LEN, wgsl::KW_MAX_LEN),
            LangId::Ada => (ada::KW_MIN_LEN, ada::KW_MAX_LEN),
            LangId::Groovy => (groovy::KW_MIN_LEN, groovy::KW_MAX_LEN),
            LangId::Clojure => (clojure::KW_MIN_LEN, clojure::KW_MAX_LEN),
            LangId::Assembly => (assembly::KW_MIN_LEN, assembly::KW_MAX_LEN),
            LangId::Tcl => (tcl::KW_MIN_LEN, tcl::KW_MAX_LEN),
            LangId::Scheme => (scheme::KW_MIN_LEN, scheme::KW_MAX_LEN),
            LangId::Json => (json::KW_MIN_LEN, json::KW_MAX_LEN),
            LangId::Toml => (toml::KW_MIN_LEN, toml::KW_MAX_LEN),
            LangId::Yaml => (yaml::KW_MIN_LEN, yaml::KW_MAX_LEN),
            LangId::Dockerfile => (dockerfile::KW_MIN_LEN, dockerfile::KW_MAX_LEN),
            LangId::Makefile => (makefile::KW_MIN_LEN, makefile::KW_MAX_LEN),
            LangId::CMake => (cmake::KW_MIN_LEN, cmake::KW_MAX_LEN),
            LangId::Glsl => (glsl::KW_MIN_LEN, glsl::KW_MAX_LEN),
            LangId::Hlsl => (hlsl::KW_MIN_LEN, hlsl::KW_MAX_LEN),
            LangId::ObjectiveC => (objc::KW_MIN_LEN, objc::KW_MAX_LEN),
            LangId::V => (v::KW_MIN_LEN, v::KW_MAX_LEN),
            LangId::CommonLisp => (clisp::KW_MIN_LEN, clisp::KW_MAX_LEN),
            LangId::Pascal => (pascal::KW_MIN_LEN, pascal::KW_MAX_LEN),
            LangId::PowerShell => (powershell::KW_MIN_LEN, powershell::KW_MAX_LEN),
            LangId::Fish => (fish::KW_MIN_LEN, fish::KW_MAX_LEN),
            LangId::Awk => (awk::KW_MIN_LEN, awk::KW_MAX_LEN),
            LangId::Hare => (hare::KW_MIN_LEN, hare::KW_MAX_LEN),
            LangId::Html => (html::KW_MIN_LEN, html::KW_MAX_LEN),
            LangId::Xml => (xml::KW_MIN_LEN, xml::KW_MAX_LEN),
            LangId::Css => (css::KW_MIN_LEN, css::KW_MAX_LEN),
            LangId::Graphql => (graphql::KW_MIN_LEN, graphql::KW_MAX_LEN),
            LangId::Protobuf => (protobuf::KW_MIN_LEN, protobuf::KW_MAX_LEN),
            LangId::Cobol => (cobol::KW_MIN_LEN, cobol::KW_MAX_LEN),
            LangId::Vhdl => (vhdl::KW_MIN_LEN, vhdl::KW_MAX_LEN),
            LangId::CHeader => (c::KW_MIN_LEN, c::KW_MAX_LEN),
            LangId::CppHeader => (cpp::KW_MIN_LEN, cpp::KW_MAX_LEN),
        }
    }

    #[cfg(test)]
    pub fn token_kind_and_lut(lang: LangId) -> (&'static [u8; 256], &'static [i8; 32]) {
        match lang {
            LangId::C => (&c::TOKEN_KIND, &c::TOKEN_LUT_LO),
            LangId::Cpp => (&cpp::TOKEN_KIND, &cpp::TOKEN_LUT_LO),
            LangId::Rust => (&rust::TOKEN_KIND, &rust::TOKEN_LUT_LO),
            LangId::Python => (&python::TOKEN_KIND, &python::TOKEN_LUT_LO),
            LangId::Perl => (&perl::TOKEN_KIND, &perl::TOKEN_LUT_LO),
            LangId::Go => (&go::TOKEN_KIND, &go::TOKEN_LUT_LO),
            LangId::Zig => (&zig::TOKEN_KIND, &zig::TOKEN_LUT_LO),
            LangId::Odin => (&odin::TOKEN_KIND, &odin::TOKEN_LUT_LO),
            LangId::JavaScript => (&javascript::TOKEN_KIND, &javascript::TOKEN_LUT_LO),
            LangId::TypeScript => (&typescript::TOKEN_KIND, &typescript::TOKEN_LUT_LO),
            LangId::Lua => (&lua::TOKEN_KIND, &lua::TOKEN_LUT_LO),
            LangId::Java => (&java::TOKEN_KIND, &java::TOKEN_LUT_LO),
            LangId::CSharp => (&csharp::TOKEN_KIND, &csharp::TOKEN_LUT_LO),
            LangId::Ruby => (&ruby::TOKEN_KIND, &ruby::TOKEN_LUT_LO),
            LangId::Php => (&php::TOKEN_KIND, &php::TOKEN_LUT_LO),
            LangId::Swift => (&swift::TOKEN_KIND, &swift::TOKEN_LUT_LO),
            LangId::Kotlin => (&kotlin::TOKEN_KIND, &kotlin::TOKEN_LUT_LO),
            LangId::Bash => (&bash::TOKEN_KIND, &bash::TOKEN_LUT_LO),
            LangId::Sql => (&sql::TOKEN_KIND, &sql::TOKEN_LUT_LO),
            LangId::Dart => (&dart::TOKEN_KIND, &dart::TOKEN_LUT_LO),
            LangId::Scala => (&scala::TOKEN_KIND, &scala::TOKEN_LUT_LO),
            LangId::Haskell => (&haskell::TOKEN_KIND, &haskell::TOKEN_LUT_LO),
            LangId::OCaml => (&ocaml::TOKEN_KIND, &ocaml::TOKEN_LUT_LO),
            LangId::Gleam => (&gleam::TOKEN_KIND, &gleam::TOKEN_LUT_LO),
            LangId::D => (&d::TOKEN_KIND, &d::TOKEN_LUT_LO),
            LangId::Nim => (&nim::TOKEN_KIND, &nim::TOKEN_LUT_LO),
            LangId::Fortran => (&fortran::TOKEN_KIND, &fortran::TOKEN_LUT_LO),
            LangId::Elixir => (&elixir::TOKEN_KIND, &elixir::TOKEN_LUT_LO),
            LangId::Erlang => (&erlang::TOKEN_KIND, &erlang::TOKEN_LUT_LO),
            LangId::Julia => (&julia::TOKEN_KIND, &julia::TOKEN_LUT_LO),
            LangId::R => (&r::TOKEN_KIND, &r::TOKEN_LUT_LO),
            LangId::Matlab => (&matlab::TOKEN_KIND, &matlab::TOKEN_LUT_LO),
            LangId::Solidity => (&solidity::TOKEN_KIND, &solidity::TOKEN_LUT_LO),
            LangId::FSharp => (&fsharp::TOKEN_KIND, &fsharp::TOKEN_LUT_LO),
            LangId::Mojo => (&mojo::TOKEN_KIND, &mojo::TOKEN_LUT_LO),
            LangId::Verilog => (&verilog::TOKEN_KIND, &verilog::TOKEN_LUT_LO),
            LangId::Cuda => (&cuda::TOKEN_KIND, &cuda::TOKEN_LUT_LO),
            LangId::Wgsl => (&wgsl::TOKEN_KIND, &wgsl::TOKEN_LUT_LO),
            LangId::Ada => (&ada::TOKEN_KIND, &ada::TOKEN_LUT_LO),
            LangId::Groovy => (&groovy::TOKEN_KIND, &groovy::TOKEN_LUT_LO),
            LangId::Clojure => (&clojure::TOKEN_KIND, &clojure::TOKEN_LUT_LO),
            LangId::Assembly => (&assembly::TOKEN_KIND, &assembly::TOKEN_LUT_LO),
            LangId::Tcl => (&tcl::TOKEN_KIND, &tcl::TOKEN_LUT_LO),
            LangId::Scheme => (&scheme::TOKEN_KIND, &scheme::TOKEN_LUT_LO),
            LangId::Json => (&json::TOKEN_KIND, &json::TOKEN_LUT_LO),
            LangId::Toml => (&toml::TOKEN_KIND, &toml::TOKEN_LUT_LO),
            LangId::Yaml => (&yaml::TOKEN_KIND, &yaml::TOKEN_LUT_LO),
            LangId::Dockerfile => (&dockerfile::TOKEN_KIND, &dockerfile::TOKEN_LUT_LO),
            LangId::Makefile => (&makefile::TOKEN_KIND, &makefile::TOKEN_LUT_LO),
            LangId::CMake => (&cmake::TOKEN_KIND, &cmake::TOKEN_LUT_LO),
            LangId::Glsl => (&glsl::TOKEN_KIND, &glsl::TOKEN_LUT_LO),
            LangId::Hlsl => (&hlsl::TOKEN_KIND, &hlsl::TOKEN_LUT_LO),
            LangId::ObjectiveC => (&objc::TOKEN_KIND, &objc::TOKEN_LUT_LO),
            LangId::V => (&v::TOKEN_KIND, &v::TOKEN_LUT_LO),
            LangId::CommonLisp => (&clisp::TOKEN_KIND, &clisp::TOKEN_LUT_LO),
            LangId::Pascal => (&pascal::TOKEN_KIND, &pascal::TOKEN_LUT_LO),
            LangId::PowerShell => (&powershell::TOKEN_KIND, &powershell::TOKEN_LUT_LO),
            LangId::Fish => (&fish::TOKEN_KIND, &fish::TOKEN_LUT_LO),
            LangId::Awk => (&awk::TOKEN_KIND, &awk::TOKEN_LUT_LO),
            LangId::Hare => (&hare::TOKEN_KIND, &hare::TOKEN_LUT_LO),
            LangId::Html => (&html::TOKEN_KIND, &html::TOKEN_LUT_LO),
            LangId::Xml => (&xml::TOKEN_KIND, &xml::TOKEN_LUT_LO),
            LangId::Css => (&css::TOKEN_KIND, &css::TOKEN_LUT_LO),
            LangId::Graphql => (&graphql::TOKEN_KIND, &graphql::TOKEN_LUT_LO),
            LangId::Protobuf => (&protobuf::TOKEN_KIND, &protobuf::TOKEN_LUT_LO),
            LangId::Cobol => (&cobol::TOKEN_KIND, &cobol::TOKEN_LUT_LO),
            LangId::Vhdl => (&vhdl::TOKEN_KIND, &vhdl::TOKEN_LUT_LO),
            LangId::CHeader => (&c::TOKEN_KIND, &c::TOKEN_LUT_LO),
            LangId::CppHeader => (&cpp::TOKEN_KIND, &cpp::TOKEN_LUT_LO),
        }
    }
}

#[inline(always)]
fn has_stem(name: &[u8], dot_idx: usize) -> bool {
    dot_idx > 0 && name[dot_idx - 1] != b'/' && name[dot_idx - 1] != b'\\'
}

#[inline(always)]
pub fn detect_language(name: &[u8], enabled_mask: u128) -> Option<LangId> {
    if (enabled_mask & LANG_MAKEFILE_MASK) != 0 && (name == b"Makefile" || name == b"makefile" || name == b"GNUmakefile") {
        return Some(LangId::Makefile);
    }
    if (enabled_mask & LANG_DOCKERFILE_MASK) != 0 && (name == b"Dockerfile" || name == b"dockerfile" || name.starts_with(b"Dockerfile.") || name.starts_with(b"dockerfile.")) {
        return Some(LangId::Dockerfile);
    }
    if (enabled_mask & LANG_CMAKE_MASK) != 0 && (name == b"CMakeLists.txt" || name == b"cmakelists.txt") {
        return Some(LangId::CMake);
    }
    let len = name.len();
    if len < 3 {
        return None;
    }
    if name[len - 2] == b'.' {
        if !has_stem(name, len - 2) {
            return None;
        }
        match name[len - 1] | 0x20 {
            b'c' if (enabled_mask & LANG_C_MASK) != 0 => return Some(LangId::C),
            b'd' if (enabled_mask & LANG_D_MASK) != 0 => return Some(LangId::D),
            b'f' if (enabled_mask & LANG_FORTRAN_MASK) != 0 => return Some(LangId::Fortran),
            b'h' => {
                if (enabled_mask & LANG_CHEADER_MASK) != 0 {
                    return Some(LangId::CHeader);
                }
                if (enabled_mask & LANG_CPPHEADER_MASK) != 0 {
                    return Some(LangId::CppHeader);
                }
            }
            b't' if (enabled_mask & LANG_PERL_MASK) != 0 => return Some(LangId::Perl),
            b'r' if (enabled_mask & LANG_R_MASK) != 0 => return Some(LangId::R),
            b'm' => {
                if (enabled_mask & LANG_MATLAB_MASK) != 0 {
                    return Some(LangId::Matlab);
                }
                if (enabled_mask & LANG_OBJECTIVEC_MASK) != 0 {
                    return Some(LangId::ObjectiveC);
                }
            }
            b'v' => {
                if (enabled_mask & LANG_VERILOG_MASK) != 0 {
                    return Some(LangId::Verilog);
                }
                if (enabled_mask & LANG_V_MASK) != 0 {
                    return Some(LangId::V);
                }
            }
            b's' if (enabled_mask & LANG_ASSEMBLY_MASK) != 0 => return Some(LangId::Assembly),
            _ => {}
        }
        return None;
    }
    if len >= 4 && name[len - 3] == b'.' {
        if !has_stem(name, len - 3) {
            return None;
        }
        let ext = [name[len - 2] | 0x20, name[len - 1] | 0x20];
        match &ext {
            b"rs" if (enabled_mask & LANG_RUST_MASK) != 0 => return Some(LangId::Rust),
            b"hs" if (enabled_mask & LANG_HASKELL_MASK) != 0 => return Some(LangId::Haskell),
            b"cc" if (enabled_mask & LANG_CPP_MASK) != 0 => return Some(LangId::Cpp),
            b"hh" if (enabled_mask & LANG_CPPHEADER_MASK) != 0 => return Some(LangId::CppHeader),
            b"py" if (enabled_mask & LANG_PYTHON_MASK) != 0 => return Some(LangId::Python),
            b"pl" | b"pm" if (enabled_mask & LANG_PERL_MASK) != 0 => return Some(LangId::Perl),
            b"go" if (enabled_mask & LANG_GO_MASK) != 0 => return Some(LangId::Go),
            b"js" if (enabled_mask & LANG_JAVASCRIPT_MASK) != 0 => {
                return Some(LangId::JavaScript);
            }
            b"ts" if (enabled_mask & LANG_TYPESCRIPT_MASK) != 0 => {
                return Some(LangId::TypeScript);
            }
            b"cs" if (enabled_mask & LANG_CSHARP_MASK) != 0 => return Some(LangId::CSharp),
            b"rb" if (enabled_mask & LANG_RUBY_MASK) != 0 => return Some(LangId::Ruby),
            b"kt" if (enabled_mask & LANG_KOTLIN_MASK) != 0 => return Some(LangId::Kotlin),
            b"sh" if (enabled_mask & LANG_BASH_MASK) != 0 => return Some(LangId::Bash),
            b"sc" if (enabled_mask & LANG_SCALA_MASK) != 0 => return Some(LangId::Scala),
            b"ml" if (enabled_mask & LANG_OCAML_MASK) != 0 => return Some(LangId::OCaml),
            b"di" if (enabled_mask & LANG_D_MASK) != 0 => return Some(LangId::D),
            b"ex" if (enabled_mask & LANG_ELIXIR_MASK) != 0 => return Some(LangId::Elixir),
            b"jl" if (enabled_mask & LANG_JULIA_MASK) != 0 => return Some(LangId::Julia),
            b"fs" if (enabled_mask & LANG_FSHARP_MASK) != 0 => return Some(LangId::FSharp),
            b"sv" | b"vh" if (enabled_mask & LANG_VERILOG_MASK) != 0 => return Some(LangId::Verilog),
            b"cu" if (enabled_mask & LANG_CUDA_MASK) != 0 => return Some(LangId::Cuda),
            b"ss" if (enabled_mask & LANG_SCHEME_MASK) != 0 => return Some(LangId::Scheme),
            b"mk" if (enabled_mask & LANG_MAKEFILE_MASK) != 0 => return Some(LangId::Makefile),
            b"mm" if (enabled_mask & LANG_OBJECTIVEC_MASK) != 0 => return Some(LangId::ObjectiveC),
            b"ha" if (enabled_mask & LANG_HARE_MASK) != 0 => return Some(LangId::Hare),
            b"cl" if (enabled_mask & LANG_COMMONLISP_MASK) != 0 => return Some(LangId::CommonLisp),
            b"pp" if (enabled_mask & LANG_PASCAL_MASK) != 0 => return Some(LangId::Pascal),
            _ => {}
        }
        return None;
    }
    if len >= 5 && name[len - 4] == b'.' {
        if !has_stem(name, len - 4) {
            return None;
        }
        let ext = [
            name[len - 3] | 0x20,
            name[len - 2] | 0x20,
            name[len - 1] | 0x20,
        ];
        match &ext {
            b"cpp" | b"cxx" | b"c++" if (enabled_mask & LANG_CPP_MASK) != 0 => {
                return Some(LangId::Cpp);
            }
            b"hpp" | b"hxx" | b"h++" | b"ipp" | b"inl" | b"ixx"
                if (enabled_mask & LANG_CPPHEADER_MASK) != 0 =>
            {
                return Some(LangId::CppHeader);
            }
            b"pyi" | b"pyw" if (enabled_mask & LANG_PYTHON_MASK) != 0 => {
                return Some(LangId::Python);
            }
            b"lua" if (enabled_mask & LANG_LUA_MASK) != 0 => return Some(LangId::Lua),
            b"lhs" if (enabled_mask & LANG_HASKELL_MASK) != 0 => return Some(LangId::Haskell),
            b"mli" if (enabled_mask & LANG_OCAML_MASK) != 0 => return Some(LangId::OCaml),
            b"zig" | b"zon" if (enabled_mask & LANG_ZIG_MASK) != 0 => return Some(LangId::Zig),
            b"mjs" | b"cjs" | b"jsx" if (enabled_mask & LANG_JAVASCRIPT_MASK) != 0 => {
                return Some(LangId::JavaScript);
            }
            b"mts" | b"cts" | b"tsx" if (enabled_mask & LANG_TYPESCRIPT_MASK) != 0 => {
                return Some(LangId::TypeScript);
            }
            b"php" if (enabled_mask & LANG_PHP_MASK) != 0 => return Some(LangId::Php),
            b"kts" if (enabled_mask & LANG_KOTLIN_MASK) != 0 => return Some(LangId::Kotlin),
            b"sql" if (enabled_mask & LANG_SQL_MASK) != 0 => return Some(LangId::Sql),
            b"nim" if (enabled_mask & LANG_NIM_MASK) != 0 => return Some(LangId::Nim),
            b"for" | b"f90" | b"f95" | b"f03" | b"f08"
                if (enabled_mask & LANG_FORTRAN_MASK) != 0 =>
            {
                return Some(LangId::Fortran);
            }
            b"exs" if (enabled_mask & LANG_ELIXIR_MASK) != 0 => return Some(LangId::Elixir),
            b"erl" | b"hrl" if (enabled_mask & LANG_ERLANG_MASK) != 0 => {
                return Some(LangId::Erlang);
            }
            b"sol" if (enabled_mask & LANG_SOLIDITY_MASK) != 0 => return Some(LangId::Solidity),
            b"fsi" | b"fsx" if (enabled_mask & LANG_FSHARP_MASK) != 0 => return Some(LangId::FSharp),
            b"svh" if (enabled_mask & LANG_VERILOG_MASK) != 0 => return Some(LangId::Verilog),
            b"cuh" if (enabled_mask & LANG_CUDA_MASK) != 0 => return Some(LangId::Cuda),
            b"adb" | b"ads" if (enabled_mask & LANG_ADA_MASK) != 0 => return Some(LangId::Ada),
            b"gvy" if (enabled_mask & LANG_GROOVY_MASK) != 0 => return Some(LangId::Groovy),
            b"clj" | b"edn" if (enabled_mask & LANG_CLOJURE_MASK) != 0 => return Some(LangId::Clojure),
            b"asm" if (enabled_mask & LANG_ASSEMBLY_MASK) != 0 => return Some(LangId::Assembly),
            b"tcl" if (enabled_mask & LANG_TCL_MASK) != 0 => return Some(LangId::Tcl),
            b"scm" | b"rkt" if (enabled_mask & LANG_SCHEME_MASK) != 0 => return Some(LangId::Scheme),
            b"yml" if (enabled_mask & LANG_YAML_MASK) != 0 => return Some(LangId::Yaml),
            b"vsh" if (enabled_mask & LANG_V_MASK) != 0 => return Some(LangId::V),
            b"lsp" if (enabled_mask & LANG_COMMONLISP_MASK) != 0 => return Some(LangId::CommonLisp),
            b"pas" if (enabled_mask & LANG_PASCAL_MASK) != 0 => return Some(LangId::Pascal),
            b"ps1" if (enabled_mask & LANG_POWERSHELL_MASK) != 0 => return Some(LangId::PowerShell),
            b"awk" if (enabled_mask & LANG_AWK_MASK) != 0 => return Some(LangId::Awk),
            b"htm" if (enabled_mask & LANG_HTML_MASK) != 0 => return Some(LangId::Html),
            b"xml" | b"svg" if (enabled_mask & LANG_XML_MASK) != 0 => return Some(LangId::Xml),
            b"css" if (enabled_mask & LANG_CSS_MASK) != 0 => return Some(LangId::Css),
            b"gql" if (enabled_mask & LANG_GRAPHQL_MASK) != 0 => return Some(LangId::Graphql),
            b"cbl" | b"cob" | b"cpy" if (enabled_mask & LANG_COBOL_MASK) != 0 => return Some(LangId::Cobol),
            b"vhd" if (enabled_mask & LANG_VHDL_MASK) != 0 => return Some(LangId::Vhdl),
            _ => {}
        }
        return None;
    }
    if len >= 6 && name[len - 5] == b'.' {
        if !has_stem(name, len - 5) {
            return None;
        }
        let ext = [
            name[len - 4] | 0x20,
            name[len - 3] | 0x20,
            name[len - 2] | 0x20,
            name[len - 1] | 0x20,
        ];
        match &ext {
            b"odin" if (enabled_mask & LANG_ODIN_MASK) != 0 => return Some(LangId::Odin),
            b"cppm" if (enabled_mask & LANG_CPP_MASK) != 0 => return Some(LangId::Cpp),
            b"java" if (enabled_mask & LANG_JAVA_MASK) != 0 => return Some(LangId::Java),
            b"bash" if (enabled_mask & LANG_BASH_MASK) != 0 => return Some(LangId::Bash),
            b"dart" if (enabled_mask & LANG_DART_MASK) != 0 => return Some(LangId::Dart),
            b"nims" if (enabled_mask & LANG_NIM_MASK) != 0 => return Some(LangId::Nim),
            b"mojo" if (enabled_mask & LANG_MOJO_MASK) != 0 => return Some(LangId::Mojo),
            b"wgsl" if (enabled_mask & LANG_WGSL_MASK) != 0 => return Some(LangId::Wgsl),
            b"cljs" | b"cljc" if (enabled_mask & LANG_CLOJURE_MASK) != 0 => return Some(LangId::Clojure),
            b"json" if (enabled_mask & LANG_JSON_MASK) != 0 => return Some(LangId::Json),
            b"toml" if (enabled_mask & LANG_TOML_MASK) != 0 => return Some(LangId::Toml),
            b"yaml" if (enabled_mask & LANG_YAML_MASK) != 0 => return Some(LangId::Yaml),
            b"glsl" | b"vert" | b"frag" | b"geom" | b"comp" if (enabled_mask & LANG_GLSL_MASK) != 0 => return Some(LangId::Glsl),
            b"hlsl" if (enabled_mask & LANG_HLSL_MASK) != 0 => return Some(LangId::Hlsl),
            b"lisp" if (enabled_mask & LANG_COMMONLISP_MASK) != 0 => return Some(LangId::CommonLisp),
            b"psm1" | b"psd1" if (enabled_mask & LANG_POWERSHELL_MASK) != 0 => return Some(LangId::PowerShell),
            b"fish" if (enabled_mask & LANG_FISH_MASK) != 0 => return Some(LangId::Fish),
            b"html" if (enabled_mask & LANG_HTML_MASK) != 0 => return Some(LangId::Html),
            b"scss" | b"sass" | b"less" if (enabled_mask & LANG_CSS_MASK) != 0 => return Some(LangId::Css),
            b"vhdl" if (enabled_mask & LANG_VHDL_MASK) != 0 => return Some(LangId::Vhdl),
            _ => {}
        }
        return None;
    }
    if len >= 7 && name[len - 6] == b'.' {
        if !has_stem(name, len - 6) {
            return None;
        }
        let ext = [
            name[len - 5] | 0x20,
            name[len - 4] | 0x20,
            name[len - 3] | 0x20,
            name[len - 2] | 0x20,
            name[len - 1] | 0x20,
        ];
        if &ext == b"phtml" && (enabled_mask & LANG_PHP_MASK) != 0 {
            return Some(LangId::Php);
        }
        if &ext == b"swift" && (enabled_mask & LANG_SWIFT_MASK) != 0 {
            return Some(LangId::Swift);
        }
        if &ext == b"scala" && (enabled_mask & LANG_SCALA_MASK) != 0 {
            return Some(LangId::Scala);
        }
        if &ext == b"gleam" && (enabled_mask & LANG_GLEAM_MASK) != 0 {
            return Some(LangId::Gleam);
        }
        if &ext == b"xhtml" && (enabled_mask & LANG_HTML_MASK) != 0 {
            return Some(LangId::Html);
        }
        if &ext == b"proto" && (enabled_mask & LANG_PROTOBUF_MASK) != 0 {
            return Some(LangId::Protobuf);
        }
    }
    if len >= 8 && name[len - 7] == b'.' {
        if !has_stem(name, len - 7) {
            return None;
        }
        let ext = [
            name[len - 6] | 0x20,
            name[len - 5] | 0x20,
            name[len - 4] | 0x20,
            name[len - 3] | 0x20,
            name[len - 2] | 0x20,
            name[len - 1] | 0x20,
        ];
        if &ext == b"nimble" && (enabled_mask & LANG_NIM_MASK) != 0 {
            return Some(LangId::Nim);
        }
        if &ext == b"groovy" && (enabled_mask & LANG_GROOVY_MASK) != 0 {
            return Some(LangId::Groovy);
        }
        if &ext == b"gradle" && (enabled_mask & LANG_GROOVY_MASK) != 0 {
            return Some(LangId::Groovy);
        }
    }
    if len >= 9 && name[len - 8] == b'.' {
        if !has_stem(name, len - 8) {
            return None;
        }
        let ext = [
            name[len - 7] | 0x20,
            name[len - 6] | 0x20,
            name[len - 5] | 0x20,
            name[len - 4] | 0x20,
            name[len - 3] | 0x20,
            name[len - 2] | 0x20,
            name[len - 1] | 0x20,
        ];
        if &ext == b"graphql" && (enabled_mask & LANG_GRAPHQL_MASK) != 0 {
            return Some(LangId::Graphql);
        }
    }
    if len >= 7 && name[len - 6] == b'.' {
        let ext = [
            name[len - 5] | 0x20,
            name[len - 4] | 0x20,
            name[len - 3] | 0x20,
            name[len - 2] | 0x20,
            name[len - 1] | 0x20,
        ];
        if &ext == b"cmake" && (enabled_mask & LANG_CMAKE_MASK) != 0 {
            return Some(LangId::CMake);
        }
    }
    None
}

#[inline(always)]
fn get_basename(w: &[u8]) -> &[u8] {
    let mut k = w.len();
    while k > 0 {
        if w[k - 1] == b'/' {
            return &w[k..];
        }
        k -= 1;
    }
    w
}

#[inline(always)]
pub fn detect_shebang(content: &[u8], enabled_mask: u128) -> Option<LangId> {
    if content.len() < 3 || content[0] != b'#' || content[1] != b'!' {
        return None;
    }

    let mut i = 2;
    while i < content.len() && (content[i] == b' ' || content[i] == b'\t') {
        i += 1;
    }

    let mut line_end = i;
    while line_end < content.len() && content[line_end] != b'\n' && content[line_end] != b'\r' {
        line_end += 1;
    }

    while line_end > i && (content[line_end - 1] == b' ' || content[line_end - 1] == b'\t') {
        line_end -= 1;
    }

    let mut words = [&b""[..]; 8];
    let mut word_count = 0;
    let mut p = i;
    while p < line_end && word_count < 8 {
        while p < line_end && (content[p] == b' ' || content[p] == b'\t') {
            p += 1;
        }
        if p >= line_end {
            break;
        }
        let start = p;
        while p < line_end && content[p] != b' ' && content[p] != b'\t' {
            p += 1;
        }
        words[word_count] = &content[start..p];
        word_count += 1;
    }

    if word_count == 0 {
        return None;
    }

    let mut prog = get_basename(words[0]);
    if prog == b"env" {
        let mut idx = 1;
        while idx < word_count && words[idx].starts_with(b"-") {
            idx += 1;
        }
        if idx < word_count {
            prog = get_basename(words[idx]);
        } else {
            return None;
        }
    }

    if (enabled_mask & LANG_BASH_MASK) != 0
        && (prog == b"bash" || prog == b"sh" || prog == b"zsh" || prog == b"dash" || prog == b"ash")
    {
        return Some(LangId::Bash);
    }
    if (enabled_mask & LANG_PYTHON_MASK) != 0 && prog.starts_with(b"python") {
        return Some(LangId::Python);
    }
    if (enabled_mask & LANG_PERL_MASK) != 0 && prog.starts_with(b"perl") {
        return Some(LangId::Perl);
    }
    if (enabled_mask & LANG_RUBY_MASK) != 0 && prog == b"ruby" {
        return Some(LangId::Ruby);
    }
    if (enabled_mask & LANG_FISH_MASK) != 0 && prog == b"fish" {
        return Some(LangId::Fish);
    }
    if (enabled_mask & LANG_AWK_MASK) != 0 && (prog == b"awk" || prog == b"gawk" || prog == b"mawk" || prog == b"nawk") {
        return Some(LangId::Awk);
    }
    if (enabled_mask & LANG_TCL_MASK) != 0 && (prog == b"tclsh" || prog == b"wish") {
        return Some(LangId::Tcl);
    }
    if (enabled_mask & LANG_JAVASCRIPT_MASK) != 0 && (prog == b"node" || prog == b"nodejs") {
        return Some(LangId::JavaScript);
    }

    None
}

pub const TOK_WS: u8 = 0;
pub const TOK_NL: u8 = 1;
pub const TOK_CODE: u8 = 2;

pub const STATE_NONE: u8 = 0;
pub const STATE_CODE: u8 = 1;
pub const STATE_COMMENT: u8 = 2;

pub const TMPL_MAX_DEPTH: usize = 32;

pub struct TemplateStack {
    pub depths: [u32; TMPL_MAX_DEPTH],
    pub len: usize,
}

impl TemplateStack {
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            depths: [0; TMPL_MAX_DEPTH],
            len: 0,
        }
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline(always)]
    pub fn open_brace(&mut self) {
        if self.len > 0 {
            self.depths[self.len - 1] += 1;
        }
    }

    #[inline(always)]
    pub fn close_brace(&mut self) -> bool {
        if self.len == 0 {
            return false;
        }
        let top = &mut self.depths[self.len - 1];
        if *top == 0 {
            self.len -= 1;
            true
        } else {
            *top -= 1;
            false
        }
    }
}

#[inline(always)]
fn scan_template_chunk(content: &[u8], mut i: usize) -> (usize, bool) {
    let len = content.len();
    while i < len {
        let Some(pos) = memchr::memchr3(b'`', b'\\', b'$', &content[i..]) else {
            return (len, false);
        };
        let p = i + pos;
        match content[p] {
            b'`' => return (p + 1, false),
            b'\\' => i = p + 2,
            _ => {
                if p + 1 < len && content[p + 1] == b'{' {
                    return (p + 2, true);
                }
                i = p + 1;
            }
        }
    }
    (len, false)
}

#[inline(always)]
pub fn skip_template(content: &[u8], start: usize, stack: &mut TemplateStack) -> usize {
    let mut i = start;
    loop {
        let (end, interp) = scan_template_chunk(content, i);
        if !interp {
            return end.min(content.len());
        }
        if stack.len < TMPL_MAX_DEPTH {
            stack.depths[stack.len] = 0;
            stack.len += 1;
            return end;
        }
        i = end;
    }
}

#[inline(always)]
pub fn finish_trailing_line<const N: usize>(
    content: &[u8],
    line_state: u8,
    acc: &mut ParseResult<N>,
) {
    if content.last().is_none_or(|&b| b == b'\n') {
        return;
    }
    if (line_state & STATE_CODE) != 0 {
        acc.code_lines += 1;
    } else if (line_state & STATE_COMMENT) != 0 {
        acc.comment_lines += 1;
    } else {
        acc.blank_lines += 1;
    }
    acc.lines_processed += 1;
}

pub trait LangLexer<const N: usize> {
    const TOKEN_KIND: [u8; 256];
    const TOKEN_LUT_LO: [i8; 32];
    const HAS_DOLLAR_IDENT: bool;
    const SUPPORTS_TEMPLATES: bool;
    const TOKEN_KIND_INTERP: [u8; 256] = Self::TOKEN_KIND;
    const TOKEN_INTERP_LUT_LO: [i8; 32] = Self::TOKEN_LUT_LO;
    const HAS_SPECIAL_IDENT: bool = false;
    const HAS_LINE_START_HOOK: bool = false;

    fn match_keyword(cand: &[u8]) -> Option<usize>;

    #[inline(always)]
    fn handle_ident(
        _content: &[u8],
        _start: usize,
        _end: usize,
        _line_state: &mut u8,
        _acc: &mut ParseResult<N>,
    ) -> usize {
        _end
    }

    #[inline(always)]
    fn handle_line_start(
        _content: &[u8],
        _i: usize,
        _line_state: &mut u8,
        _acc: &mut ParseResult<N>,
    ) -> usize {
        _i
    }

    fn handle_special(
        kind: u8,
        content: &[u8],
        i: usize,
        line_state: &mut u8,
        acc: &mut ParseResult<N>,
    ) -> usize;
}

pub fn unified_parse_into<const N: usize, L: LangLexer<N>>(
    content: &[u8],
    acc: &mut ParseResult<N>,
) {
    let content_len = content.len();
    if content_len == 0 {
        return;
    }

    let mut tmpl = TemplateStack::new();
    let mut line_state = STATE_NONE;
    let mut i = 0usize;
    while i < content_len {
        if line_state == STATE_NONE {
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
            if L::HAS_LINE_START_HOOK {
                let next_i = L::handle_line_start(content, i, &mut line_state, acc);
                if next_i != i {
                    i = next_i;
                    continue;
                }
            }
        }

        let in_interp = L::SUPPORTS_TEMPLATES && !tmpl.is_empty();
        let kinds = if in_interp {
            &L::TOKEN_KIND_INTERP
        } else {
            &L::TOKEN_KIND
        };

        if kinds[content[i] as usize] <= TOK_CODE && content[i] != b'\n' {
            line_state |= STATE_CODE;
            i += 1;
            if i < content_len
                && kinds[content[i] as usize] <= TOK_CODE
                && content[i] != b'\n'
            {
                i = if in_interp {
                    skip_non_tokens_with_lut_avx2(
                        content,
                        i,
                        &L::TOKEN_KIND_INTERP,
                        &L::TOKEN_INTERP_LUT_LO,
                    )
                } else {
                    skip_non_tokens_with_lut_avx2(
                        content,
                        i,
                        &L::TOKEN_KIND,
                        &L::TOKEN_LUT_LO,
                    )
                };
            }
            if i >= content_len {
                break;
            }
        }

        let b = content[i];
        let kind = kinds[b as usize];
        match kind {
            TOK_NL => {
                if (line_state & STATE_CODE) != 0 {
                    acc.code_lines += 1;
                } else if (line_state & STATE_COMMENT) != 0 {
                    acc.comment_lines += 1;
                } else {
                    acc.blank_lines += 1;
                }
                acc.lines_processed += 1;
                line_state = STATE_NONE;
                i += 1;
            }
            3 => {
                line_state |= STATE_CODE;
                let start = i;
                let end = if L::HAS_DOLLAR_IDENT {
                    skip_js_ident_cont_avx2(content, i + 1)
                } else {
                    skip_ident_cont_avx2(content, i + 1)
                };
                if L::HAS_SPECIAL_IDENT {
                    let next_i = L::handle_ident(content, start, end, &mut line_state, acc);
                    if next_i != end {
                        i = next_i;
                        continue;
                    }
                }
                i = end;
                if let Some(idx) = L::match_keyword(&content[start..end]) {
                    acc.keyword_counts[idx] += 1;
                    acc.total_keywords += 1;
                }
            }
            7 if L::SUPPORTS_TEMPLATES => {
                line_state |= STATE_CODE;
                let end_i = skip_template(content, i + 1, &mut tmpl);
                let nl = count_newlines_avx2(&content[i + 1..end_i]);
                if nl > 0 {
                    acc.code_lines += nl as i64;
                    acc.lines_processed += nl as i64;
                }
                i = end_i;
            }
            8 if L::SUPPORTS_TEMPLATES => {
                line_state |= STATE_CODE;
                tmpl.open_brace();
                i += 1;
            }
            9 if L::SUPPORTS_TEMPLATES => {
                line_state |= STATE_CODE;
                if tmpl.close_brace() {
                    let end_i = skip_template(content, i + 1, &mut tmpl);
                    let nl = count_newlines_avx2(&content[i + 1..end_i]);
                    if nl > 0 {
                        acc.code_lines += nl as i64;
                        acc.lines_processed += nl as i64;
                    }
                    i = end_i;
                } else {
                    i += 1;
                }
            }
            _ => {
                i = L::handle_special(kind, content, i, &mut line_state, acc);
            }
        }
    }
    finish_trailing_line(content, line_state, acc);
}

pub const fn compute_keyword_lens<const N: usize>(names: &[&str; N]) -> [u8; N] {
    assert!(N > 0 && N < 255);
    let mut lens = [0u8; N];
    let mut i = 0;
    while i < N {
        let a = names[i].as_bytes();
        assert!(a.len() >= 2 && a.len() <= 16);
        lens[i] = a.len() as u8;
        let mut j = i + 1;
        while j < N {
            let b = names[j].as_bytes();
            if a.len() == b.len() {
                let mut eq = true;
                let mut k = 0;
                while k < a.len() {
                    if a[k] != b[k] {
                        eq = false;
                        break;
                    }
                    k += 1;
                }
                assert!(!eq);
            }
            j += 1;
        }
        i += 1;
    }
    lens
}

pub const fn compute_kw_min_len<const N: usize>(names: &[&str; N]) -> usize {
    let mut min = usize::MAX;
    let mut i = 0;
    while i < N {
        let len = names[i].len();
        if len < min {
            min = len;
        }
        i += 1;
    }
    assert!(min >= 2);
    min
}

pub const fn compute_kw_max_len<const N: usize>(names: &[&str; N]) -> usize {
    let mut max = 0;
    let mut i = 0;
    while i < N {
        let len = names[i].len();
        if len > max {
            max = len;
        }
        i += 1;
    }
    assert!(max >= 2 && max <= 16);
    max
}

pub const AVX2_ASCII_LUT_HI: [i8; 32] = [
    1, 2, 4, 8, 16, 32, 64, -128, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 4, 8, 16, 32, 64, -128, 0, 0, 0,
    0, 0, 0, 0, 0,
];

pub const fn make_token_lut_lo(token_kind: &[u8; 256]) -> [i8; 32] {
    let mut lo16 = [0u8; 16];
    let mut b = 0usize;
    while b < 256 {
        let is_token = token_kind[b] > TOK_CODE || b == b'\n' as usize;
        if is_token {
            assert!(b < 128);
            lo16[b & 0x0F] |= 1u8 << (b >> 4);
        }
        b += 1;
    }
    let mut out = [0i8; 32];
    let mut i = 0usize;
    while i < 16 {
        out[i] = lo16[i] as i8;
        out[i + 16] = lo16[i] as i8;
        i += 1;
    }
    let mut check = 0usize;
    while check < 256 {
        let is_token = token_kind[check] > TOK_CODE || check == b'\n' as usize;
        let lo = if check < 128 { out[check & 0x0F] } else { 0 };
        let hi = AVX2_ASCII_LUT_HI[check >> 4];
        let hit = (lo & hi) != 0;
        assert!(hit == is_token);
        check += 1;
    }
    out
}

pub const fn make_ident_lut_lo(ident_cont: &[u8; 256]) -> [i8; 32] {
    let mut lo16 = [0u8; 16];
    let mut b = 0usize;
    while b < 256 {
        let is_ident = ident_cont[b] != 0;
        if is_ident {
            assert!(b < 128);
            lo16[b & 0x0F] |= 1u8 << (b >> 4);
        }
        b += 1;
    }
    let mut out = [0i8; 32];
    let mut i = 0usize;
    while i < 16 {
        out[i] = lo16[i] as i8;
        out[i + 16] = lo16[i] as i8;
        i += 1;
    }
    let mut check = 0usize;
    while check < 256 {
        let is_ident = ident_cont[check] != 0;
        let lo = if check < 128 { out[check & 0x0F] } else { 0 };
        let hi = AVX2_ASCII_LUT_HI[check >> 4];
        let hit = (lo & hi) != 0;
        assert!(hit == is_ident);
        check += 1;
    }
    out
}

pub const IDENT_CONT: [u8; 256] = {
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

pub const IDENT_LUT_LO: [i8; 32] = make_ident_lut_lo(&IDENT_CONT);

pub const JS_IDENT_CONT: [u8; 256] = {
    let mut table = IDENT_CONT;
    table[b'$' as usize] = 1;
    table
};

pub const JS_IDENT_LUT_LO: [i8; 32] = make_ident_lut_lo(&JS_IDENT_CONT);

#[inline(always)]
pub fn skip_non_tokens_with_lut_avx2(
    ptr: &[u8],
    mut i: usize,
    token_kind: &[u8; 256],
    lut_lo_arr: &[i8; 32],
) -> usize {
    debug_assert!(i <= ptr.len());
    #[cfg(target_feature = "avx2")]
    {
        use safe_arch::*;
        let lut_lo = m256i::from(*lut_lo_arr);
        let lut_hi = m256i::from(AVX2_ASCII_LUT_HI);
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
    while i < ptr.len() && token_kind[ptr[i] as usize] <= TOK_CODE && ptr[i] != b'\n' {
        i += 1;
    }
    i
}

#[inline(always)]
pub fn skip_ident_cont_with_lut_avx2(
    ptr: &[u8],
    mut i: usize,
    ident_cont: &[u8; 256],
    lut_lo_arr: &[i8; 32],
) -> usize {
    debug_assert!(i <= ptr.len());
    #[cfg(target_feature = "avx2")]
    {
        use safe_arch::*;
        let lut_lo = m256i::from(*lut_lo_arr);
        let lut_hi = m256i::from(AVX2_ASCII_LUT_HI);
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
    while i < ptr.len() && ident_cont[ptr[i] as usize] != 0 {
        i += 1;
    }
    i
}

#[inline(always)]
pub fn skip_ident_cont_avx2(ptr: &[u8], i: usize) -> usize {
    skip_ident_cont_with_lut_avx2(ptr, i, &IDENT_CONT, &IDENT_LUT_LO)
}

#[inline(always)]
pub fn skip_js_ident_cont_avx2(ptr: &[u8], i: usize) -> usize {
    skip_ident_cont_with_lut_avx2(ptr, i, &JS_IDENT_CONT, &JS_IDENT_LUT_LO)
}

#[inline(always)]
pub fn kw_match_2_to_16(cand: &[u8], kw: &[u8]) -> bool {
    let n = cand.len();
    if n != kw.len() || !(2..=16).contains(&n) {
        return false;
    }
    if n >= 8 {
        let a1 = u64::from_ne_bytes(cand[..8].try_into().unwrap());
        let b1 = u64::from_ne_bytes(kw[..8].try_into().unwrap());
        let a2 = u64::from_ne_bytes(cand[n - 8..n].try_into().unwrap());
        let b2 = u64::from_ne_bytes(kw[n - 8..n].try_into().unwrap());
        ((a1 ^ b1) | (a2 ^ b2)) == 0
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
pub fn skip_nested_block_comment(content: &[u8], start_i: usize) -> usize {
    assert!(start_i <= content.len());
    let content_len = content.len();
    let Some(first_close) = find_star_slash(&content[start_i..]) else {
        return content_len;
    };
    let mut i = start_i;
    let first_end = start_i + first_close;
    let mut has_nested = false;
    let mut k = start_i;
    while k + 1 < first_end {
        let Some(pos) = find_byte_avx2(&content[k..first_end - 1], b'/') else {
            break;
        };
        if content[k + pos + 1] == b'*' {
            has_nested = true;
            break;
        }
        k += pos + 1;
    }
    if !has_nested {
        let res = first_end + 2;
        assert!(res <= content_len);
        return res;
    }

    let mut depth = 1usize;
    while i + 1 < content_len {
        let b0 = content[i];
        let b1 = content[i + 1];
        if b0 == b'/' && b1 == b'*' {
            depth += 1;
            i += 2;
        } else if b0 == b'*' && b1 == b'/' {
            depth -= 1;
            i += 2;
            if depth == 0 {
                assert!(i <= content_len);
                return i;
            }
        } else {
            i += 1;
        }
    }
    content_len
}

#[inline(always)]
pub fn skip_nested_plus_slash_comment(content: &[u8], start_i: usize) -> usize {
    let content_len = content.len();
    let mut i = start_i;
    let mut depth = 1usize;
    while i + 1 < content_len {
        let b0 = content[i];
        let b1 = content[i + 1];
        if b0 == b'/' && b1 == b'+' {
            depth += 1;
            i += 2;
        } else if b0 == b'+' && b1 == b'/' {
            depth -= 1;
            i += 2;
            if depth == 0 {
                return i;
            }
        } else {
            i += 1;
        }
    }
    content_len
}

#[inline(always)]
pub fn skip_nested_hash_bracket_comment(content: &[u8], start_i: usize) -> usize {
    let content_len = content.len();
    let mut i = start_i;
    let mut depth = 1usize;
    while i + 1 < content_len {
        let b0 = content[i];
        let b1 = content[i + 1];
        if b0 == b'#' && b1 == b'[' {
            depth += 1;
            i += 2;
        } else if b0 == b']' && b1 == b'#' {
            depth -= 1;
            i += 2;
            if depth == 0 {
                return i;
            }
        } else {
            i += 1;
        }
    }
    content_len
}

#[inline(always)]
pub fn skip_nested_hash_equal_comment(content: &[u8], start_i: usize) -> usize {
    let content_len = content.len();
    let mut i = start_i;
    let mut depth = 1usize;
    while i + 1 < content_len {
        let b0 = content[i];
        let b1 = content[i + 1];
        if b0 == b'#' && b1 == b'=' {
            depth += 1;
            i += 2;
        } else if b0 == b'=' && b1 == b'#' {
            depth -= 1;
            i += 2;
            if depth == 0 {
                return i;
            }
        } else {
            i += 1;
        }
    }
    content_len
}

#[inline(always)]
pub fn skip_triple_literal(content: &[u8], start_i: usize, quote: u8) -> usize {
    assert!(start_i <= content.len());
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
                let res = q_idx + 3;
                assert!(res <= content_len);
                return res;
            }
        }
        i = q_idx + 1;
    }
    content_len
}

#[inline(always)]
pub fn parse_into(lang: LangId, content: &[u8], stats: &mut LangStats) {
    match lang {
        LangId::C => c::parse_into(content, &mut stats.c),
        LangId::Cpp => cpp::parse_into(content, &mut stats.cpp),
        LangId::Rust => rust::parse_into(content, &mut stats.rust),
        LangId::Python => python::parse_into(content, &mut stats.python),
        LangId::Perl => perl::parse_into(content, &mut stats.perl),
        LangId::Go => go::parse_into(content, &mut stats.go),
        LangId::Zig => zig::parse_into(content, &mut stats.zig),
        LangId::Odin => odin::parse_into(content, &mut stats.odin),
        LangId::JavaScript => javascript::parse_into(content, &mut stats.javascript),
        LangId::TypeScript => typescript::parse_into(content, &mut stats.typescript),
        LangId::Lua => lua::parse_into(content, &mut stats.lua),
        LangId::Java => java::parse_into(content, &mut stats.java),
        LangId::CSharp => csharp::parse_into(content, &mut stats.csharp),
        LangId::Ruby => ruby::parse_into(content, &mut stats.ruby),
        LangId::Php => php::parse_into(content, &mut stats.php),
        LangId::Swift => swift::parse_into(content, &mut stats.swift),
        LangId::Kotlin => kotlin::parse_into(content, &mut stats.kotlin),
        LangId::Bash => bash::parse_into(content, &mut stats.bash),
        LangId::Sql => sql::parse_into(content, &mut stats.sql),
        LangId::Dart => dart::parse_into(content, &mut stats.dart),
        LangId::Scala => scala::parse_into(content, &mut stats.scala),
        LangId::Haskell => haskell::parse_into(content, &mut stats.haskell),
        LangId::OCaml => ocaml::parse_into(content, &mut stats.ocaml),
        LangId::Gleam => gleam::parse_into(content, &mut stats.gleam),
        LangId::D => d::parse_into(content, &mut stats.d),
        LangId::Nim => nim::parse_into(content, &mut stats.nim),
        LangId::Fortran => fortran::parse_into(content, &mut stats.fortran),
        LangId::Elixir => elixir::parse_into(content, &mut stats.elixir),
        LangId::Erlang => erlang::parse_into(content, &mut stats.erlang),
        LangId::Julia => julia::parse_into(content, &mut stats.julia),
        LangId::R => r::parse_into(content, &mut stats.r),
        LangId::Matlab => matlab::parse_into(content, &mut stats.matlab),
        LangId::Solidity => solidity::parse_into(content, &mut stats.solidity),
        LangId::FSharp => fsharp::parse_into(content, &mut stats.fsharp),
        LangId::Mojo => mojo::parse_into(content, &mut stats.mojo),
        LangId::Verilog => verilog::parse_into(content, &mut stats.verilog),
        LangId::Cuda => cuda::parse_into(content, &mut stats.cuda),
        LangId::Wgsl => wgsl::parse_into(content, &mut stats.wgsl),
        LangId::Ada => ada::parse_into(content, &mut stats.ada),
        LangId::Groovy => groovy::parse_into(content, &mut stats.groovy),
        LangId::Clojure => clojure::parse_into(content, &mut stats.clojure),
        LangId::Assembly => assembly::parse_into(content, &mut stats.assembly),
        LangId::Tcl => tcl::parse_into(content, &mut stats.tcl),
        LangId::Scheme => scheme::parse_into(content, &mut stats.scheme),
        LangId::Json => json::parse_into(content, &mut stats.json),
        LangId::Toml => toml::parse_into(content, &mut stats.toml),
        LangId::Yaml => yaml::parse_into(content, &mut stats.yaml),
        LangId::Dockerfile => dockerfile::parse_into(content, &mut stats.dockerfile),
        LangId::Makefile => makefile::parse_into(content, &mut stats.makefile),
        LangId::CMake => cmake::parse_into(content, &mut stats.cmake),
        LangId::Glsl => glsl::parse_into(content, &mut stats.glsl),
        LangId::Hlsl => hlsl::parse_into(content, &mut stats.hlsl),
        LangId::ObjectiveC => objc::parse_into(content, &mut stats.objc),
        LangId::V => v::parse_into(content, &mut stats.v),
        LangId::CommonLisp => clisp::parse_into(content, &mut stats.clisp),
        LangId::Pascal => pascal::parse_into(content, &mut stats.pascal),
        LangId::PowerShell => powershell::parse_into(content, &mut stats.powershell),
        LangId::Fish => fish::parse_into(content, &mut stats.fish),
        LangId::Awk => awk::parse_into(content, &mut stats.awk),
        LangId::Hare => hare::parse_into(content, &mut stats.hare),
        LangId::Html => html::parse_into(content, &mut stats.html),
        LangId::Xml => xml::parse_into(content, &mut stats.xml),
        LangId::Css => css::parse_into(content, &mut stats.css),
        LangId::Graphql => graphql::parse_into(content, &mut stats.graphql),
        LangId::Protobuf => protobuf::parse_into(content, &mut stats.protobuf),
        LangId::Cobol => cobol::parse_into(content, &mut stats.cobol),
        LangId::Vhdl => vhdl::parse_into(content, &mut stats.vhdl),
        LangId::CHeader => c::parse_into(content, &mut stats.c_header),
        LangId::CppHeader => cpp::parse_into(content, &mut stats.cpp_header),
    }
}

#[inline(always)]
pub fn count_newlines_avx2(bytes: &[u8]) -> usize {
    #[cfg(target_feature = "avx2")]
    {
        use safe_arch::*;
        let target = set_splat_i8_m256i(b'\n' as i8);
        let zero = zeroed_m256i();
        let mut count = 0usize;
        let (chunks, remainder) = bytes.as_chunks::<32>();
        for block in chunks.chunks(255) {
            let mut acc = zeroed_m256i();
            for &chunk in block {
                let v = m256i::from(chunk);
                let cmp = cmp_eq_mask_i8_m256i(v, target);
                acc = sub_i8_m256i(acc, cmp);
            }
            let sad: [u64; 4] = sum_of_u8_abs_diff_m256i(acc, zero).into();
            count += (sad[0] + sad[1] + sad[2] + sad[3]) as usize;
        }
        for &b in remainder {
            if b == b'\n' {
                count += 1;
            }
        }
        count
    }
    #[cfg(not(target_feature = "avx2"))]
    {
        memchr::memchr_iter(b'\n', bytes).count()
    }
}

#[inline(always)]
pub fn find_byte_avx2(slice: &[u8], needle: u8) -> Option<usize> {
    #[cfg(target_feature = "avx2")]
    {
        use safe_arch::*;
        let target = set_splat_i8_m256i(needle as i8);
        let (chunks, remainder) = slice.as_chunks::<32>();
        let mut i = 0usize;
        for &chunk in chunks {
            let v = m256i::from(chunk);
            let cmp = cmp_eq_mask_i8_m256i(v, target);
            let mask = move_mask_i8_m256i(cmp) as u32;
            if mask != 0 {
                return Some(i + mask.trailing_zeros() as usize);
            }
            i += 32;
        }
        for (idx, &b) in remainder.iter().enumerate() {
            if b == needle {
                return Some(i + idx);
            }
        }
        None
    }
    #[cfg(not(target_feature = "avx2"))]
    {
        memchr::memchr(needle, slice)
    }
}

#[inline(always)]
pub fn find_star_slash(slice: &[u8]) -> Option<usize> {
    if slice.len() < 2 {
        return None;
    }
    #[cfg(target_feature = "avx2")]
    {
        use safe_arch::*;
        let stars = set_splat_i8_m256i(b'*' as i8);
        let slashes = set_splat_i8_m256i(b'/' as i8);
        let mut i = 0usize;
        let (chunks0, _) = slice[..slice.len() - 1].as_chunks::<32>();
        let (chunks1, _) = slice[1..].as_chunks::<32>();
        for (&c0_arr, &c1_arr) in chunks0.iter().zip(chunks1.iter()) {
            let c0 = m256i::from(c0_arr);
            let c1 = m256i::from(c1_arr);
            let eq_star = cmp_eq_mask_i8_m256i(c0, stars);
            let eq_slash = cmp_eq_mask_i8_m256i(c1, slashes);
            let mask = move_mask_i8_m256i(eq_star & eq_slash) as u32;
            if mask != 0 {
                return Some(i + mask.trailing_zeros() as usize);
            }
            i += 32;
        }
        while i + 1 < slice.len() {
            if slice[i] == b'*' && slice[i + 1] == b'/' {
                return Some(i);
            }
            i += 1;
        }
        None
    }
    #[cfg(not(target_feature = "avx2"))]
    {
        memchr::memmem::find(slice, b"*/")
    }
}

#[allow(dead_code)]
#[inline(always)]
pub fn skip_line_comment(content: &[u8], start_i: usize) -> usize {
    debug_assert!(start_i <= content.len());
    let content_len = content.len();
    let mut i = start_i;
    while i < content_len {
        let Some(pos) = find_byte_avx2(&content[i..], b'\n') else {
            return content_len;
        };
        let nl_idx = i + pos;
        let mut check = nl_idx;
        if check > start_i && content[check - 1] == b'\r' {
            check -= 1;
        }
        if check > start_i && content[check - 1] == b'\\' {
            i = nl_idx + 1;
            continue;
        }
        return nl_idx + 1;
    }
    content_len
}

#[inline(always)]
pub fn skip_literal(content: &[u8], start_i: usize, quote: u8) -> usize {
    assert!(start_i <= content.len());
    assert!(quote == b'"' || quote == b'\'' || quote == b'`');
    let content_len = content.len();
    let mut i = start_i;
    while i < content_len {
        let Some(pos) = find_byte_avx2(&content[i..], quote) else {
            return content_len;
        };
        i += pos;
        let mut escapes = 0usize;
        let mut j = i;
        while j > start_i && content[j - 1] == b'\\' {
            escapes += 1;
            j -= 1;
        }
        if (escapes & 1) == 0 {
            let res = i + 1;
            assert!(res <= content_len);
            return res;
        }
        i += 1;
    }
    content_len
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line_metrics(lang: LangId, content: &[u8]) -> [i64; 4] {
        let mut stats = LangStats::default();
        parse_into(lang, content, &mut stats);
        [
            stats.lines(lang),
            stats.code_lines(lang),
            stats.comment_lines(lang),
            stats.blank_lines(lang),
        ]
    }

    #[test]
    fn test_final_line_without_newline_all_languages() {
        for lang in ALL_LANGS {
            let cmt: &str = match lang {
                LangId::Python
                | LangId::Perl
                | LangId::Ruby
                | LangId::Bash
                | LangId::Nim
                | LangId::Elixir
                | LangId::Julia
                | LangId::R
                | LangId::Mojo
                | LangId::Tcl
                | LangId::Toml
                | LangId::Yaml
                | LangId::Dockerfile
                | LangId::Makefile
                | LangId::CMake => "#",
                LangId::Lua | LangId::Sql | LangId::Haskell | LangId::Ada => "--",
                LangId::OCaml => "(*",
                LangId::Fortran => "!",
                LangId::Erlang | LangId::Matlab => "%",
                LangId::Clojure | LangId::Scheme | LangId::Assembly | LangId::CommonLisp => ";",
                LangId::Fish | LangId::Awk | LangId::PowerShell | LangId::Graphql => "#",
                LangId::Vhdl => "--",
                LangId::Cobol => "*>",
                LangId::Html | LangId::Xml => "<!--",
                LangId::Json => "//",
                _ => "//",
            };
            let mut samples: Vec<String> = vec![
                "a\nb".to_string(),
                "a\n\nb".to_string(),
                "\"x\ny\"".to_string(),
                "a\n   ".to_string(),
                "a".to_string(),
            ];
            if lang != LangId::Json {
                samples.push(format!("a\n{cmt} c"));
                samples.push(format!("a\nb {cmt} c"));
                samples.push(format!("{cmt} c"));
            }
            if !matches!(
                lang,
                LangId::Python
                    | LangId::Perl
                    | LangId::Zig
                    | LangId::Lua
                    | LangId::Ruby
                    | LangId::Bash
                    | LangId::Haskell
                    | LangId::OCaml
                    | LangId::Gleam
                    | LangId::Nim
                    | LangId::Fortran
                    | LangId::Elixir
                    | LangId::Erlang
                    | LangId::Julia
                    | LangId::R
                    | LangId::Matlab
                    | LangId::Mojo
                    | LangId::FSharp
                    | LangId::Ada
                    | LangId::Clojure
                    | LangId::Tcl
                    | LangId::Scheme
                    | LangId::Json
                    | LangId::Toml
                    | LangId::Yaml
                    | LangId::Dockerfile
                    | LangId::Makefile
                    | LangId::CMake
                    | LangId::CommonLisp
                    | LangId::Fish
                    | LangId::Awk
                    | LangId::Html
                    | LangId::Xml
                    | LangId::Graphql
                    | LangId::Cobol
                    | LangId::Vhdl
                    | LangId::PowerShell
                    | LangId::Pascal
            ) {
                samples.push("a\n/* x\ny */".to_string());
                samples.push("a /* x\ny".to_string());
                samples.push("a\n/* x */ b".to_string());
            }
            if lang == LangId::Scheme || lang == LangId::CommonLisp {
                samples.push("a\n#| x\ny |#".to_string());
                samples.push("a #| x\ny".to_string());
                samples.push("a\n#| x |# b".to_string());
            }
            if lang == LangId::Haskell {
                samples.push("a\n{- x\ny -}".to_string());
                samples.push("a {- x\ny".to_string());
                samples.push("a\n{- x -} b".to_string());
            }
            if lang == LangId::OCaml || lang == LangId::FSharp || lang == LangId::Pascal {
                samples.push("a\n(* x\ny *)".to_string());
                samples.push("a (* x\ny".to_string());
                samples.push("a\n(* x *) b".to_string());
            }
            if lang == LangId::Pascal {
                samples.push("a\n{ x\ny }".to_string());
                samples.push("a { x\ny".to_string());
                samples.push("a\n{ x } b".to_string());
            }
            if lang == LangId::PowerShell {
                samples.push("a\n<# x\ny #>".to_string());
                samples.push("a <# x\ny".to_string());
                samples.push("a\n<# x #> b".to_string());
            }
            if lang == LangId::Html || lang == LangId::Xml {
                samples.push("a\n<!-- x\ny -->".to_string());
                samples.push("a <!-- x\ny".to_string());
                samples.push("a\n<!-- x --> b".to_string());
            }
            if lang == LangId::Lua {
                samples.push("a\n--[[ x\ny ]]".to_string());
                samples.push("a --[[ x\ny".to_string());
                samples.push("a\n--[[ x ]] b".to_string());
            }
            if lang == LangId::Perl {
                samples.push("a\n=pod\nx\ny".to_string());
                samples.push("a\n__END__\nx\ny".to_string());
            }
            for s in &samples {
                let without = line_metrics(lang, s.as_bytes());
                let with = line_metrics(lang, format!("{s}\n").as_bytes());
                assert_eq!(without, with, "{} sample {:?}", lang.name(), s);
                assert_eq!(
                    without[0],
                    without[1] + without[2] + without[3],
                    "{} sample {:?}",
                    lang.name(),
                    s
                );
            }
            assert_eq!(line_metrics(lang, b"a\nb"), [2, 2, 0, 0], "{}", lang.name());
            if lang != LangId::Json {
                assert_eq!(
                    line_metrics(lang, format!("a\n{cmt} c").as_bytes()),
                    [2, 1, 1, 0],
                    "{}",
                    lang.name()
                );
            }
        }
    }

    #[test]
    fn test_count_newlines_saturation_and_boundaries() {
        assert_eq!(count_newlines_avx2(b""), 0);
        assert_eq!(count_newlines_avx2(b"a"), 0);
        assert_eq!(count_newlines_avx2(b"\n"), 1);

        for len in [1, 31, 32, 33, 63, 64, 65, 8159, 8160, 8161, 17000] {
            let all_nl = vec![b'\n'; len];
            assert_eq!(count_newlines_avx2(&all_nl), len);

            let no_nl = vec![b'x'; len];
            assert_eq!(count_newlines_avx2(&no_nl), 0);
        }
    }

    #[test]
    fn test_find_byte_and_star_slash_boundaries() {
        assert_eq!(find_byte_avx2(b"", b'x'), None);
        assert_eq!(find_star_slash(b""), None);
        assert_eq!(find_star_slash(b"*"), None);
        assert_eq!(find_star_slash(b"/"), None);
        assert_eq!(find_star_slash(b"/*"), None);

        for total_len in [2, 31, 32, 33, 63, 64, 65, 96, 100] {
            for pos in 0..total_len {
                let mut buf = vec![b'.'; total_len];
                buf[pos] = b'!';
                assert_eq!(find_byte_avx2(&buf, b'!'), Some(pos));
                assert_eq!(find_byte_avx2(&buf, b'?'), None);
            }

            for pos in 0..total_len - 1 {
                let mut buf = vec![b'.'; total_len];
                buf[pos] = b'*';
                buf[pos + 1] = b'/';
                assert_eq!(find_star_slash(&buf), Some(pos));
            }
        }
    }

    #[test]
    fn test_skip_comment_and_literal_edge_cases() {
        assert_eq!(skip_line_comment(b"no newline at all", 0), 17);
        assert_eq!(skip_line_comment(b"trailing backslash \\", 0), 20);
        assert_eq!(skip_line_comment(b"spliced \\\nstill comment\nnext", 0), 24);
        assert_eq!(
            skip_line_comment(b"spliced crlf \\\r\nstill comment\r\nnext", 0),
            31
        );

        assert_eq!(skip_literal(b"unclosed string", 0, b'"'), 15);
        assert_eq!(skip_literal(b"trailing escape \\", 0, b'"'), 17);
        assert_eq!(skip_literal(b"\\\\\"rest", 0, b'"'), 3);
        assert_eq!(skip_literal(b"\\\\\\\"\"rest", 0, b'"'), 5);

        for offset in 28..36 {
            let mut s = vec![b'a'; 80];
            s[offset] = b'\\';
            s[offset + 1] = b'"';
            s[offset + 10] = b'"';
            assert_eq!(skip_literal(&s, 0, b'"'), offset + 11);
        }
    }

    #[test]
    fn test_detect_language_and_header_routing() {
        assert_eq!(detect_language(b"foo.c", LANG_ALL_MASK), Some(LangId::C));
        assert_eq!(detect_language(b"foo.h", LANG_ALL_MASK), Some(LangId::CHeader));
        assert_eq!(detect_language(b"foo.H", LANG_ALL_MASK), Some(LangId::CHeader));
        assert_eq!(
            detect_language(b"foo.cpp", LANG_ALL_MASK),
            Some(LangId::Cpp)
        );
        assert_eq!(
            detect_language(b"foo.hpp", LANG_ALL_MASK),
            Some(LangId::CppHeader)
        );
        assert_eq!(
            detect_language(b"foo.rs", LANG_ALL_MASK),
            Some(LangId::Rust)
        );
        assert_eq!(
            detect_language(b"foo.py", LANG_ALL_MASK),
            Some(LangId::Python)
        );
        assert_eq!(
            detect_language(b"foo.pl", LANG_ALL_MASK),
            Some(LangId::Perl)
        );
        assert_eq!(
            detect_language(b"foo.pm", LANG_ALL_MASK),
            Some(LangId::Perl)
        );
        assert_eq!(
            detect_language(b"foo.t", LANG_ALL_MASK),
            Some(LangId::Perl)
        );
        assert_eq!(
            detect_language(b"foo.go", LANG_ALL_MASK),
            Some(LangId::Go)
        );
        assert_eq!(
            detect_language(b"foo.zig", LANG_ALL_MASK),
            Some(LangId::Zig)
        );
        assert_eq!(
            detect_language(b"foo.zon", LANG_ALL_MASK),
            Some(LangId::Zig)
        );
        assert_eq!(
            detect_language(b"foo.odin", LANG_ALL_MASK),
            Some(LangId::Odin)
        );
        assert_eq!(
            detect_language(b"foo.js", LANG_ALL_MASK),
            Some(LangId::JavaScript)
        );
        assert_eq!(
            detect_language(b"foo.mjs", LANG_ALL_MASK),
            Some(LangId::JavaScript)
        );
        assert_eq!(
            detect_language(b"foo.cjs", LANG_ALL_MASK),
            Some(LangId::JavaScript)
        );
        assert_eq!(
            detect_language(b"foo.jsx", LANG_ALL_MASK),
            Some(LangId::JavaScript)
        );
        assert_eq!(
            detect_language(b"foo.ts", LANG_ALL_MASK),
            Some(LangId::TypeScript)
        );
        assert_eq!(
            detect_language(b"foo.mts", LANG_ALL_MASK),
            Some(LangId::TypeScript)
        );
        assert_eq!(
            detect_language(b"foo.cts", LANG_ALL_MASK),
            Some(LangId::TypeScript)
        );
        assert_eq!(
            detect_language(b"foo.tsx", LANG_ALL_MASK),
            Some(LangId::TypeScript)
        );
        assert_eq!(detect_language(b"foo.rb", LANG_ALL_MASK), Some(LangId::Ruby));
        assert_eq!(detect_language(b"foo.php", LANG_ALL_MASK), Some(LangId::Php));
        assert_eq!(
            detect_language(b"foo.phtml", LANG_ALL_MASK),
            Some(LangId::Php)
        );
        assert_eq!(
            detect_language(b"foo.swift", LANG_ALL_MASK),
            Some(LangId::Swift)
        );
        assert_eq!(
            detect_language(b"foo.kt", LANG_ALL_MASK),
            Some(LangId::Kotlin)
        );
        assert_eq!(
            detect_language(b"foo.kts", LANG_ALL_MASK),
            Some(LangId::Kotlin)
        );
        assert_eq!(
            detect_language(b"foo.sh", LANG_ALL_MASK),
            Some(LangId::Bash)
        );
        assert_eq!(
            detect_language(b"foo.bash", LANG_ALL_MASK),
            Some(LangId::Bash)
        );
        assert_eq!(
            detect_language(b"foo.sql", LANG_ALL_MASK),
            Some(LangId::Sql)
        );
        assert_eq!(
            detect_language(b"foo.dart", LANG_ALL_MASK),
            Some(LangId::Dart)
        );
        assert_eq!(
            detect_language(b"foo.scala", LANG_ALL_MASK),
            Some(LangId::Scala)
        );
        assert_eq!(
            detect_language(b"foo.sc", LANG_ALL_MASK),
            Some(LangId::Scala)
        );
        assert_eq!(
            detect_language(b"foo.hs", LANG_ALL_MASK),
            Some(LangId::Haskell)
        );
        assert_eq!(
            detect_language(b"foo.lhs", LANG_ALL_MASK),
            Some(LangId::Haskell)
        );
        assert_eq!(
            detect_language(b"foo.ml", LANG_ALL_MASK),
            Some(LangId::OCaml)
        );
        assert_eq!(
            detect_language(b"foo.mli", LANG_ALL_MASK),
            Some(LangId::OCaml)
        );
        assert_eq!(
            detect_language(b"foo.gleam", LANG_ALL_MASK),
            Some(LangId::Gleam)
        );
        assert_eq!(
            detect_language(b"foo.sol", LANG_ALL_MASK),
            Some(LangId::Solidity)
        );
        assert_eq!(
            detect_language(b"foo.fs", LANG_ALL_MASK),
            Some(LangId::FSharp)
        );
        assert_eq!(
            detect_language(b"foo.fsi", LANG_ALL_MASK),
            Some(LangId::FSharp)
        );
        assert_eq!(
            detect_language(b"foo.fsx", LANG_ALL_MASK),
            Some(LangId::FSharp)
        );
        assert_eq!(
            detect_language(b"foo.mojo", LANG_ALL_MASK),
            Some(LangId::Mojo)
        );
        assert_eq!(
            detect_language(b"foo.v", LANG_ALL_MASK),
            Some(LangId::Verilog)
        );
        assert_eq!(
            detect_language(b"foo.sv", LANG_ALL_MASK),
            Some(LangId::Verilog)
        );
        assert_eq!(
            detect_language(b"foo.vh", LANG_ALL_MASK),
            Some(LangId::Verilog)
        );
        assert_eq!(
            detect_language(b"foo.svh", LANG_ALL_MASK),
            Some(LangId::Verilog)
        );
        assert_eq!(
            detect_language(b"foo.cu", LANG_ALL_MASK),
            Some(LangId::Cuda)
        );
        assert_eq!(
            detect_language(b"foo.cuh", LANG_ALL_MASK),
            Some(LangId::Cuda)
        );
        assert_eq!(
            detect_language(b"foo.wgsl", LANG_ALL_MASK),
            Some(LangId::Wgsl)
        );
        assert_eq!(
            detect_language(b"foo.adb", LANG_ALL_MASK),
            Some(LangId::Ada)
        );
        assert_eq!(
            detect_language(b"foo.ads", LANG_ALL_MASK),
            Some(LangId::Ada)
        );
        assert_eq!(
            detect_language(b"foo.groovy", LANG_ALL_MASK),
            Some(LangId::Groovy)
        );
        assert_eq!(
            detect_language(b"foo.gvy", LANG_ALL_MASK),
            Some(LangId::Groovy)
        );
        assert_eq!(
            detect_language(b"foo.gradle", LANG_ALL_MASK),
            Some(LangId::Groovy)
        );
        assert_eq!(
            detect_language(b"foo.clj", LANG_ALL_MASK),
            Some(LangId::Clojure)
        );
        assert_eq!(
            detect_language(b"foo.cljs", LANG_ALL_MASK),
            Some(LangId::Clojure)
        );
        assert_eq!(
            detect_language(b"foo.cljc", LANG_ALL_MASK),
            Some(LangId::Clojure)
        );
        assert_eq!(
            detect_language(b"foo.edn", LANG_ALL_MASK),
            Some(LangId::Clojure)
        );
        assert_eq!(
            detect_language(b"foo.s", LANG_ALL_MASK),
            Some(LangId::Assembly)
        );
        assert_eq!(
            detect_language(b"foo.S", LANG_ALL_MASK),
            Some(LangId::Assembly)
        );
        assert_eq!(
            detect_language(b"foo.asm", LANG_ALL_MASK),
            Some(LangId::Assembly)
        );
        assert_eq!(
            detect_language(b"foo.tcl", LANG_ALL_MASK),
            Some(LangId::Tcl)
        );
        assert_eq!(
            detect_language(b"foo.scm", LANG_ALL_MASK),
            Some(LangId::Scheme)
        );
        assert_eq!(
            detect_language(b"foo.ss", LANG_ALL_MASK),
            Some(LangId::Scheme)
        );
        assert_eq!(
            detect_language(b"foo.rkt", LANG_ALL_MASK),
            Some(LangId::Scheme)
        );
        assert_eq!(
            detect_language(b"foo.json", LANG_ALL_MASK),
            Some(LangId::Json)
        );
        assert_eq!(
            detect_language(b"foo.toml", LANG_ALL_MASK),
            Some(LangId::Toml)
        );
        assert_eq!(
            detect_language(b"foo.yaml", LANG_ALL_MASK),
            Some(LangId::Yaml)
        );
        assert_eq!(
            detect_language(b"foo.yml", LANG_ALL_MASK),
            Some(LangId::Yaml)
        );
        assert_eq!(
            detect_language(b"Dockerfile", LANG_ALL_MASK),
            Some(LangId::Dockerfile)
        );
        assert_eq!(
            detect_language(b"Dockerfile.build", LANG_ALL_MASK),
            Some(LangId::Dockerfile)
        );
        assert_eq!(
            detect_language(b"Makefile", LANG_ALL_MASK),
            Some(LangId::Makefile)
        );
        assert_eq!(
            detect_language(b"makefile", LANG_ALL_MASK),
            Some(LangId::Makefile)
        );
        assert_eq!(
            detect_language(b"foo.mk", LANG_ALL_MASK),
            Some(LangId::Makefile)
        );
        assert_eq!(
            detect_language(b"CMakeLists.txt", LANG_ALL_MASK),
            Some(LangId::CMake)
        );
        assert_eq!(
            detect_language(b"foo.cmake", LANG_ALL_MASK),
            Some(LangId::CMake)
        );
        assert_eq!(
            detect_language(b"foo.glsl", LANG_ALL_MASK),
            Some(LangId::Glsl)
        );
        assert_eq!(
            detect_language(b"foo.vert", LANG_ALL_MASK),
            Some(LangId::Glsl)
        );
        assert_eq!(
            detect_language(b"foo.frag", LANG_ALL_MASK),
            Some(LangId::Glsl)
        );
        assert_eq!(
            detect_language(b"foo.geom", LANG_ALL_MASK),
            Some(LangId::Glsl)
        );
        assert_eq!(
            detect_language(b"foo.comp", LANG_ALL_MASK),
            Some(LangId::Glsl)
        );
        assert_eq!(
            detect_language(b"foo.hlsl", LANG_ALL_MASK),
            Some(LangId::Hlsl)
        );
        assert_eq!(
            detect_language(b"foo.m", LANG_OBJECTIVEC_MASK),
            Some(LangId::ObjectiveC)
        );
        assert_eq!(
            detect_language(b"foo.m", LANG_MATLAB_MASK),
            Some(LangId::Matlab)
        );
        assert_eq!(
            detect_language(b"foo.mm", LANG_ALL_MASK),
            Some(LangId::ObjectiveC)
        );

        assert_eq!(detect_language(b"foo.c", LANG_C_MASK), Some(LangId::C));
        assert_eq!(detect_language(b"foo.h", LANG_CHEADER_MASK), Some(LangId::CHeader));
        assert_eq!(detect_language(b"foo.cpp", LANG_C_MASK), None);
        assert_eq!(detect_language(b"foo.hpp", LANG_C_MASK), None);
        assert_eq!(detect_language(b"foo.py", LANG_C_MASK), None);
        assert_eq!(detect_language(b"foo.pl", LANG_C_MASK), None);
        assert_eq!(detect_language(b"foo.go", LANG_C_MASK), None);

        assert_eq!(detect_language(b"foo.c", LANG_CPP_MASK), None);
        assert_eq!(detect_language(b"foo.h", LANG_CPPHEADER_MASK), Some(LangId::CppHeader));
        assert_eq!(detect_language(b"foo.H", LANG_CPPHEADER_MASK), Some(LangId::CppHeader));
        assert_eq!(
            detect_language(b"foo.cpp", LANG_CPP_MASK),
            Some(LangId::Cpp)
        );
        assert_eq!(
            detect_language(b"foo.hpp", LANG_CPPHEADER_MASK),
            Some(LangId::CppHeader)
        );
        assert_eq!(detect_language(b"foo.py", LANG_CPP_MASK), None);
        assert_eq!(detect_language(b"foo.pl", LANG_CPP_MASK), None);
        assert_eq!(detect_language(b"foo.go", LANG_CPP_MASK), None);

        assert_eq!(detect_language(b"foo.py", LANG_PYTHON_MASK), Some(LangId::Python));
        assert_eq!(detect_language(b"foo.c", LANG_PYTHON_MASK), None);

        assert_eq!(detect_language(b"foo.pl", LANG_PERL_MASK), Some(LangId::Perl));
        assert_eq!(detect_language(b"foo.pm", LANG_PERL_MASK), Some(LangId::Perl));
        assert_eq!(detect_language(b"foo.t", LANG_PERL_MASK), Some(LangId::Perl));
        assert_eq!(detect_language(b"foo.c", LANG_PERL_MASK), None);

        assert_eq!(detect_language(b"foo.go", LANG_GO_MASK), Some(LangId::Go));
        assert_eq!(detect_language(b"foo.c", LANG_GO_MASK), None);

        assert_eq!(detect_language(b"foo.zig", LANG_ZIG_MASK), Some(LangId::Zig));
        assert_eq!(detect_language(b"foo.zon", LANG_ZIG_MASK), Some(LangId::Zig));
        assert_eq!(detect_language(b"foo.c", LANG_ZIG_MASK), None);

        assert_eq!(detect_language(b"foo.odin", LANG_ODIN_MASK), Some(LangId::Odin));
        assert_eq!(detect_language(b"foo.c", LANG_ODIN_MASK), None);

        assert_eq!(detect_language(b"foo.c", 0), None);
        assert_eq!(detect_language(b"foo.h", 0), None);
        assert_eq!(detect_language(b"foo.cpp", 0), None);

        assert_eq!(detect_shebang(b"#!/bin/bash\necho 1", LANG_ALL_MASK), Some(LangId::Bash));
        assert_eq!(detect_shebang(b"#!/bin/sh\necho 1", LANG_ALL_MASK), Some(LangId::Bash));
        assert_eq!(detect_shebang(b"#!/usr/bin/env python3\nprint(1)", LANG_ALL_MASK), Some(LangId::Python));
        assert_eq!(detect_shebang(b"#!/usr/bin/env -S python3.11\nprint(1)", LANG_ALL_MASK), Some(LangId::Python));
        assert_eq!(detect_shebang(b"#!/usr/bin/perl\nprint 1;", LANG_ALL_MASK), Some(LangId::Perl));
        assert_eq!(detect_shebang(b"#!/usr/bin/ruby\nputs 1", LANG_ALL_MASK), Some(LangId::Ruby));
        assert_eq!(detect_shebang(b"#!/usr/bin/fish\necho 1", LANG_ALL_MASK), Some(LangId::Fish));
        assert_eq!(detect_shebang(b"#!/usr/bin/awk -f\nBEGIN{}", LANG_ALL_MASK), Some(LangId::Awk));
        assert_eq!(detect_shebang(b"#!/usr/bin/tclsh\nputs 1", LANG_ALL_MASK), Some(LangId::Tcl));
        assert_eq!(detect_shebang(b"#!/usr/bin/env node\nconsole.log(1)", LANG_ALL_MASK), Some(LangId::JavaScript));
        assert_eq!(detect_shebang(b"hello world\n", LANG_ALL_MASK), None);
        assert_eq!(detect_shebang(b"#!", LANG_ALL_MASK), None);
        assert_eq!(detect_shebang(b"#!/bin/bash", LANG_PYTHON_MASK), None);

        assert_eq!(detect_language(b"foo.v", LANG_ALL_MASK), Some(LangId::Verilog));
        assert_eq!(detect_language(b"foo.v", LANG_V_MASK), Some(LangId::V));
        assert_eq!(detect_language(b"foo.vsh", LANG_ALL_MASK), Some(LangId::V));
        assert_eq!(detect_language(b"foo.lisp", LANG_ALL_MASK), Some(LangId::CommonLisp));
        assert_eq!(detect_language(b"foo.lsp", LANG_ALL_MASK), Some(LangId::CommonLisp));
        assert_eq!(detect_language(b"foo.cl", LANG_ALL_MASK), Some(LangId::CommonLisp));
        assert_eq!(detect_language(b"foo.pas", LANG_ALL_MASK), Some(LangId::Pascal));
        assert_eq!(detect_language(b"foo.pp", LANG_ALL_MASK), Some(LangId::Pascal));
        assert_eq!(detect_language(b"foo.inc", LANG_ALL_MASK), None);
        assert_eq!(detect_language(b"foo.ps1", LANG_ALL_MASK), Some(LangId::PowerShell));
        assert_eq!(detect_language(b"foo.psm1", LANG_ALL_MASK), Some(LangId::PowerShell));
        assert_eq!(detect_language(b"foo.psd1", LANG_ALL_MASK), Some(LangId::PowerShell));
        assert_eq!(detect_language(b"foo.fish", LANG_ALL_MASK), Some(LangId::Fish));
        assert_eq!(detect_language(b"foo.awk", LANG_ALL_MASK), Some(LangId::Awk));
        assert_eq!(detect_language(b"foo.ha", LANG_ALL_MASK), Some(LangId::Hare));
        assert_eq!(detect_language(b"foo.html", LANG_ALL_MASK), Some(LangId::Html));
        assert_eq!(detect_language(b"foo.htm", LANG_ALL_MASK), Some(LangId::Html));
        assert_eq!(detect_language(b"foo.xhtml", LANG_ALL_MASK), Some(LangId::Html));
        assert_eq!(detect_language(b"foo.xml", LANG_ALL_MASK), Some(LangId::Xml));
        assert_eq!(detect_language(b"foo.svg", LANG_ALL_MASK), Some(LangId::Xml));
        assert_eq!(detect_language(b"foo.css", LANG_ALL_MASK), Some(LangId::Css));
        assert_eq!(detect_language(b"foo.scss", LANG_ALL_MASK), Some(LangId::Css));
        assert_eq!(detect_language(b"foo.sass", LANG_ALL_MASK), Some(LangId::Css));
        assert_eq!(detect_language(b"foo.less", LANG_ALL_MASK), Some(LangId::Css));
        assert_eq!(detect_language(b"foo.gql", LANG_ALL_MASK), Some(LangId::Graphql));
        assert_eq!(detect_language(b"foo.graphql", LANG_ALL_MASK), Some(LangId::Graphql));
        assert_eq!(detect_language(b"foo.proto", LANG_ALL_MASK), Some(LangId::Protobuf));
        assert_eq!(detect_language(b"foo.cbl", LANG_ALL_MASK), Some(LangId::Cobol));
        assert_eq!(detect_language(b"foo.cob", LANG_ALL_MASK), Some(LangId::Cobol));
        assert_eq!(detect_language(b"foo.cpy", LANG_ALL_MASK), Some(LangId::Cobol));
        assert_eq!(detect_language(b"foo.vhd", LANG_ALL_MASK), Some(LangId::Vhdl));
        assert_eq!(detect_language(b"foo.vhdl", LANG_ALL_MASK), Some(LangId::Vhdl));
    }

    #[test]
    fn test_all_languages_compile_time_and_simd_invariants() {
        for b in 0..=255u8 {
            for pos in [0, 1, 15, 16, 19, 31, 32, 33] {
                let mut block = [b'a'; 64];
                block[pos] = b;
                let id_pos = skip_ident_cont_avx2(&block, 0);
                if IDENT_CONT[b as usize] == 0 {
                    assert_eq!(id_pos, pos);
                } else {
                    assert_eq!(id_pos, block.len());
                }

                let js_id_pos = skip_js_ident_cont_avx2(&block, 0);
                if JS_IDENT_CONT[b as usize] == 0 {
                    assert_eq!(js_id_pos, pos);
                } else {
                    assert_eq!(js_id_pos, block.len());
                }
            }
        }

        for lang in ALL_LANGS {
            let names = LangStats::keyword_names(lang);
            let lens = LangStats::keyword_lens(lang);
            let (min_len, max_len) = LangStats::kw_min_max(lang);
            let (token_kind, token_lut_lo) = LangStats::token_kind_and_lut(lang);

            assert_eq!(names.len(), lens.len());
            assert!(min_len >= 2 && min_len <= max_len && max_len <= 16);

            let actual_min = names.iter().map(|s| s.len()).min().unwrap();
            let actual_max = names.iter().map(|s| s.len()).max().unwrap();
            assert_eq!(min_len, actual_min);
            assert_eq!(max_len, actual_max);

            for b in 0..=255u8 {
                for pos in [0, 1, 15, 16, 19, 31, 32, 33] {
                    let mut block = [b' '; 64];
                    block[pos] = b;
                    let skipped =
                        skip_non_tokens_with_lut_avx2(&block, 0, token_kind, token_lut_lo);
                    if token_kind[b as usize] > TOK_CODE || b == b'\n' {
                        assert_eq!(skipped, pos, "lang {:?} byte {}", lang, b);
                    } else {
                        assert_eq!(skipped, block.len(), "lang {:?} byte {}", lang, b);
                    }
                }
            }

            for (i, &kw) in names.iter().enumerate() {
                assert_eq!(kw.len(), lens[i] as usize);
                for (j, &other) in names.iter().enumerate() {
                    if i != j {
                        assert_ne!(kw, other);
                    }
                }

                let mut avx_src = Vec::with_capacity(64);
                avx_src.extend_from_slice(b"0                                    ");
                avx_src.extend_from_slice(kw.as_bytes());
                avx_src.push(b'\n');

                let mut stats = LangStats::default();
                parse_into(lang, &avx_src, &mut stats);
                assert_eq!(stats.total_keywords(lang), 1, "lang {:?} kw {}", lang, kw);
                assert_eq!(
                    stats.keyword_counts(lang)[i],
                    1,
                    "lang {:?} kw {}",
                    lang,
                    kw
                );
                assert_eq!(stats.code_lines(lang), 1);

                let mut prefixed = Vec::with_capacity(64);
                prefixed.extend_from_slice(b"0                                    _");
                prefixed.extend_from_slice(kw.as_bytes());
                prefixed.push(b'_');
                prefixed.push(b'\n');

                let mut stats2 = LangStats::default();
                parse_into(lang, &prefixed, &mut stats2);
                assert_eq!(stats2.total_keywords(lang), 0, "lang {:?} kw {}", lang, kw);
            }
        }
    }

    #[test]
    fn test_all_languages_truncation_no_panic_and_empty_inputs() {
        for lang in ALL_LANGS {
            let mut empty_stats = LangStats::default();
            parse_into(lang, b"", &mut empty_stats);
            assert_eq!(empty_stats.lines(lang), 0);
            assert_eq!(empty_stats.code_lines(lang), 0);
            assert_eq!(empty_stats.comment_lines(lang), 0);
            assert_eq!(empty_stats.blank_lines(lang), 0);
            assert_eq!(empty_stats.total_keywords(lang), 0);

            for ws in [b"   ".as_slice(), b"\t\t".as_slice(), b"\n".as_slice(), b"  \n  \n\t\n".as_slice(), b"\r\n\r\n".as_slice()] {
                let mut ws_stats = LangStats::default();
                parse_into(lang, ws, &mut ws_stats);
                assert_eq!(ws_stats.code_lines(lang), 0, "lang {:?}", lang);
                assert_eq!(ws_stats.comment_lines(lang), 0, "lang {:?}", lang);
                assert_eq!(ws_stats.total_keywords(lang), 0, "lang {:?}", lang);
                assert_eq!(ws_stats.lines(lang), ws_stats.blank_lines(lang), "lang {:?}", lang);
            }

            let cmt: &str = match lang {
                LangId::Python
                | LangId::Perl
                | LangId::Ruby
                | LangId::Bash
                | LangId::Nim
                | LangId::Elixir
                | LangId::Julia
                | LangId::R
                | LangId::Mojo
                | LangId::Tcl
                | LangId::Toml
                | LangId::Yaml
                | LangId::Dockerfile
                | LangId::Makefile
                | LangId::CMake
                | LangId::Fish
                | LangId::Awk
                | LangId::PowerShell
                | LangId::Graphql => "#",
                LangId::Lua | LangId::Sql | LangId::Haskell | LangId::Ada | LangId::Vhdl => "--",
                LangId::OCaml => "(*",
                LangId::Fortran => "!",
                LangId::Erlang | LangId::Matlab => "%",
                LangId::Clojure | LangId::Scheme | LangId::Assembly | LangId::CommonLisp => ";",
                LangId::Cobol => "*>",
                LangId::Html | LangId::Xml => "<!--",
                LangId::Json => "",
                _ => "//",
            };

            let block_open = match lang {
                LangId::Haskell => "{-",
                LangId::OCaml | LangId::FSharp | LangId::Pascal => "(*",
                LangId::Lua => "--[[",
                LangId::Scheme | LangId::CommonLisp => "#|",
                LangId::Html | LangId::Xml => "<!--",
                LangId::PowerShell => "<#",
                LangId::Fortran
                | LangId::Bash
                | LangId::Nim
                | LangId::Elixir
                | LangId::Erlang
                | LangId::Julia
                | LangId::R
                | LangId::Matlab
                | LangId::Python
                | LangId::Ruby
                | LangId::Mojo
                | LangId::Ada
                | LangId::Clojure
                | LangId::Tcl
                | LangId::Json
                | LangId::Toml
                | LangId::Yaml
                | LangId::Dockerfile
                | LangId::Makefile
                | LangId::CMake
                | LangId::Fish
                | LangId::Awk
                | LangId::Cobol
                | LangId::Vhdl => "",
                _ => "/*",
            };
            let block_close = match lang {
                LangId::Haskell => "-}",
                LangId::OCaml | LangId::FSharp | LangId::Pascal => "*)",
                LangId::Lua => "]]",
                LangId::Scheme | LangId::CommonLisp => "|#",
                LangId::Html | LangId::Xml => "-->",
                LangId::PowerShell => "#>",
                LangId::Fortran
                | LangId::Bash
                | LangId::Nim
                | LangId::Elixir
                | LangId::Erlang
                | LangId::Julia
                | LangId::R
                | LangId::Matlab
                | LangId::Python
                | LangId::Ruby
                | LangId::Mojo
                | LangId::Ada
                | LangId::Clojure
                | LangId::Tcl
                | LangId::Json
                | LangId::Toml
                | LangId::Yaml
                | LangId::Dockerfile
                | LangId::Makefile
                | LangId::CMake
                | LangId::Fish
                | LangId::Awk
                | LangId::Cobol
                | LangId::Vhdl => "",
                _ => "*/",
            };

            let first_kw = LangStats::keyword_names(lang)[0];
            let sample = if lang == LangId::Json {
                format!(
                    "{first_kw}\n\
                     \"hello \\\"world\\\" \\n\"\n\
                     {first_kw}\n\
                     \n\
                     {first_kw}\n"
                )
            } else {
                format!(
                    "{first_kw} foo = \"hello \\\"world\\\" \\n\"\n\
                     {cmt} single line comment here\n\
                     {first_kw} bar(x) {{ return x + 1; }}\n\
                     {block_open} multi line\n\
                     comment block\n\
                     {block_close}\n\
                     \n\
                     {first_kw} baz = 'c';\n"
                )
            };

            let sample_bytes = sample.as_bytes();
            for offset in 0..=sample_bytes.len() {
                let slice = &sample_bytes[..offset];
                let mut stats = LangStats::default();
                parse_into(lang, slice, &mut stats);
                let lines = stats.lines(lang);
                let code = stats.code_lines(lang);
                let comm = stats.comment_lines(lang);
                let blnk = stats.blank_lines(lang);
                assert_eq!(
                    lines,
                    code + comm + blnk,
                    "lang {:?} at offset {offset}/{}: lines={lines}, code={code}, comm={comm}, blnk={blnk}",
                    lang,
                    sample_bytes.len()
                );
            }
        }
    }
}
