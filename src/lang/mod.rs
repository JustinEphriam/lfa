pub mod bash;
pub mod c;
pub mod cpp;
pub mod csharp;
pub mod d;
pub mod dart;
pub mod gleam;
pub mod go;
pub mod haskell;
pub mod java;
pub mod javascript;
pub mod kotlin;
pub mod lua;
pub mod ocaml;
pub mod odin;
pub mod perl;
pub mod php;
pub mod python;
pub mod ruby;
pub mod rust;
pub mod scala;
pub mod sql;
pub mod swift;
pub mod typescript;
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
}

pub const LANG_COUNT: usize = 25;
pub const LANG_C_MASK: u32 = 1 << (LangId::C as u32);
pub const LANG_CPP_MASK: u32 = 1 << (LangId::Cpp as u32);
pub const LANG_RUST_MASK: u32 = 1 << (LangId::Rust as u32);
pub const LANG_PYTHON_MASK: u32 = 1 << (LangId::Python as u32);
pub const LANG_PERL_MASK: u32 = 1 << (LangId::Perl as u32);
pub const LANG_GO_MASK: u32 = 1 << (LangId::Go as u32);
pub const LANG_ZIG_MASK: u32 = 1 << (LangId::Zig as u32);
pub const LANG_ODIN_MASK: u32 = 1 << (LangId::Odin as u32);
pub const LANG_JAVASCRIPT_MASK: u32 = 1 << (LangId::JavaScript as u32);
pub const LANG_TYPESCRIPT_MASK: u32 = 1 << (LangId::TypeScript as u32);
pub const LANG_LUA_MASK: u32 = 1 << (LangId::Lua as u32);
pub const LANG_JAVA_MASK: u32 = 1 << (LangId::Java as u32);
pub const LANG_CSHARP_MASK: u32 = 1 << (LangId::CSharp as u32);
pub const LANG_RUBY_MASK: u32 = 1 << (LangId::Ruby as u32);
pub const LANG_PHP_MASK: u32 = 1 << (LangId::Php as u32);
pub const LANG_SWIFT_MASK: u32 = 1 << (LangId::Swift as u32);
pub const LANG_KOTLIN_MASK: u32 = 1 << (LangId::Kotlin as u32);
pub const LANG_BASH_MASK: u32 = 1 << (LangId::Bash as u32);
pub const LANG_SQL_MASK: u32 = 1 << (LangId::Sql as u32);
pub const LANG_DART_MASK: u32 = 1 << (LangId::Dart as u32);
pub const LANG_SCALA_MASK: u32 = 1 << (LangId::Scala as u32);
pub const LANG_HASKELL_MASK: u32 = 1 << (LangId::Haskell as u32);
pub const LANG_OCAML_MASK: u32 = 1 << (LangId::OCaml as u32);
pub const LANG_GLEAM_MASK: u32 = 1 << (LangId::Gleam as u32);
pub const LANG_D_MASK: u32 = 1 << (LangId::D as u32);
pub const LANG_ALL_MASK: u32 = (1 << LANG_COUNT) - 1;

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
];

const _: () = assert!(LANG_COUNT > 0 && LANG_COUNT <= 32);
const _: () = assert!(ALL_LANGS.len() == LANG_COUNT);

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
        }
    }

    #[inline(always)]
    pub const fn mask(self) -> u32 {
        1 << (self as u32)
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
        }
    }
}

#[inline(always)]
fn has_stem(name: &[u8], dot_idx: usize) -> bool {
    dot_idx > 0 && name[dot_idx - 1] != b'/' && name[dot_idx - 1] != b'\\'
}

#[inline(always)]
pub fn detect_language(name: &[u8], enabled_mask: u32) -> Option<LangId> {
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
            b'h' => {
                if (enabled_mask & LANG_C_MASK) != 0 {
                    return Some(LangId::C);
                }
                if (enabled_mask & LANG_CPP_MASK) != 0 {
                    return Some(LangId::Cpp);
                }
            }
            b't' if (enabled_mask & LANG_PERL_MASK) != 0 => return Some(LangId::Perl),
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
            b"cc" | b"hh" if (enabled_mask & LANG_CPP_MASK) != 0 => return Some(LangId::Cpp),
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
            b"cpp" | b"cxx" | b"c++" | b"hpp" | b"hxx" | b"h++" | b"ipp" | b"inl" | b"ixx"
                if (enabled_mask & LANG_CPP_MASK) != 0 =>
            {
                return Some(LangId::Cpp);
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
        return first_end + 2;
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
pub fn skip_triple_literal(content: &[u8], start_i: usize, quote: u8) -> usize {
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
    debug_assert!(start_i <= content.len());
    debug_assert!(quote == b'"' || quote == b'\'' || quote == b'`');
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
            return i + 1;
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
                LangId::Python | LangId::Perl | LangId::Ruby | LangId::Bash => "#",
                LangId::Lua | LangId::Sql | LangId::Haskell => "--",
                LangId::OCaml => "(*",
                _ => "//",
            };
            let mut samples: Vec<String> = vec![
                "a\nb".to_string(),
                format!("a\n{cmt} c"),
                format!("a\nb {cmt} c"),
                "a\n\nb".to_string(),
                "\"x\ny\"".to_string(),
                "a\n   ".to_string(),
                "a".to_string(),
                format!("{cmt} c"),
            ];
            if !matches!(lang, LangId::Python | LangId::Perl | LangId::Zig | LangId::Lua | LangId::Ruby | LangId::Bash | LangId::Haskell | LangId::OCaml | LangId::Gleam) {
                samples.push("a\n/* x\ny */".to_string());
                samples.push("a /* x\ny".to_string());
                samples.push("a\n/* x */ b".to_string());
            }
            if lang == LangId::Haskell {
                samples.push("a\n{- x\ny -}".to_string());
                samples.push("a {- x\ny".to_string());
                samples.push("a\n{- x -} b".to_string());
            }
            if lang == LangId::OCaml {
                samples.push("a\n(* x\ny *)".to_string());
                samples.push("a (* x\ny".to_string());
                samples.push("a\n(* x *) b".to_string());
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
            assert_eq!(
                line_metrics(lang, format!("a\n{cmt} c").as_bytes()),
                [2, 1, 1, 0],
                "{}",
                lang.name()
            );
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
        assert_eq!(detect_language(b"foo.h", LANG_ALL_MASK), Some(LangId::C));
        assert_eq!(detect_language(b"foo.H", LANG_ALL_MASK), Some(LangId::C));
        assert_eq!(
            detect_language(b"foo.cpp", LANG_ALL_MASK),
            Some(LangId::Cpp)
        );
        assert_eq!(
            detect_language(b"foo.hpp", LANG_ALL_MASK),
            Some(LangId::Cpp)
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

        assert_eq!(detect_language(b"foo.c", LANG_C_MASK), Some(LangId::C));
        assert_eq!(detect_language(b"foo.h", LANG_C_MASK), Some(LangId::C));
        assert_eq!(detect_language(b"foo.cpp", LANG_C_MASK), None);
        assert_eq!(detect_language(b"foo.hpp", LANG_C_MASK), None);
        assert_eq!(detect_language(b"foo.py", LANG_C_MASK), None);
        assert_eq!(detect_language(b"foo.pl", LANG_C_MASK), None);
        assert_eq!(detect_language(b"foo.go", LANG_C_MASK), None);

        assert_eq!(detect_language(b"foo.c", LANG_CPP_MASK), None);
        assert_eq!(detect_language(b"foo.h", LANG_CPP_MASK), Some(LangId::Cpp));
        assert_eq!(detect_language(b"foo.H", LANG_CPP_MASK), Some(LangId::Cpp));
        assert_eq!(
            detect_language(b"foo.cpp", LANG_CPP_MASK),
            Some(LangId::Cpp)
        );
        assert_eq!(
            detect_language(b"foo.hpp", LANG_CPP_MASK),
            Some(LangId::Cpp)
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
}
