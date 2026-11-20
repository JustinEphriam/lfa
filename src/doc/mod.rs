#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DocFormatId {
    Markdown = 0,
    PlainText = 1,
    Csv = 2,
    Svg = 3,
    Ini = 4,
    RestructuredText = 5,
    Patch = 6,
    Json5 = 7,
    AsciiDoc = 8,
    Org = 9,
    BibTeX = 10,
    Properties = 11,
    Gettext = 12,
    Roff = 13,
    Typst = 14,
    Djot = 15,
    Textile = 16,
    Gemtext = 17,
    Pod = 18,
    MediaWiki = 19,
    NdJson = 20,
    Avro = 21,
    Nginx = 22,
    Apache = 23,
    SshConfig = 24,
    Crontab = 25,
    GitIgnore = 26,
    Dot = 27,
    EditorConfig = 28,
    Http = 29,
    WebVtt = 30,
    Srt = 31,
    Requirements = 32,
    PlantUML = 33,
    Mermaid = 34,
    Hosts = 35,
    Fstab = 36,
    Resolv = 37,
    Log = 38,
    Xsd = 39,
    Wsdl = 40,
    CMakeCache = 41,
    Cff = 42,
    GitConfig = 43,
    DockerCompose = 44,
    Cabal = 45,
    SubStationAlpha = 46,
    Sami = 47,
    ICalendar = 48,
    TeXinfo = 49,
    GeoJson = 50,
    Kml = 51,
    RssAtom = 52,
    Xslt = 53,
    Xcode = 54,
    Autoconf = 55,
    Doxygen = 56,
    Sudoers = 57,
    Passwd = 58,
    KnownHosts = 59,
    MesonWrap = 60,
    FontForge = 61,
    GitCommit = 62,
}

pub const DOC_FORMAT_COUNT: usize = 63;
pub const ALL_DOC_FORMATS: [DocFormatId; DOC_FORMAT_COUNT] = [
    DocFormatId::Markdown,
    DocFormatId::PlainText,
    DocFormatId::Csv,
    DocFormatId::Svg,
    DocFormatId::Ini,
    DocFormatId::RestructuredText,
    DocFormatId::Patch,
    DocFormatId::Json5,
    DocFormatId::AsciiDoc,
    DocFormatId::Org,
    DocFormatId::BibTeX,
    DocFormatId::Properties,
    DocFormatId::Gettext,
    DocFormatId::Roff,
    DocFormatId::Typst,
    DocFormatId::Djot,
    DocFormatId::Textile,
    DocFormatId::Gemtext,
    DocFormatId::Pod,
    DocFormatId::MediaWiki,
    DocFormatId::NdJson,
    DocFormatId::Avro,
    DocFormatId::Nginx,
    DocFormatId::Apache,
    DocFormatId::SshConfig,
    DocFormatId::Crontab,
    DocFormatId::GitIgnore,
    DocFormatId::Dot,
    DocFormatId::EditorConfig,
    DocFormatId::Http,
    DocFormatId::WebVtt,
    DocFormatId::Srt,
    DocFormatId::Requirements,
    DocFormatId::PlantUML,
    DocFormatId::Mermaid,
    DocFormatId::Hosts,
    DocFormatId::Fstab,
    DocFormatId::Resolv,
    DocFormatId::Log,
    DocFormatId::Xsd,
    DocFormatId::Wsdl,
    DocFormatId::CMakeCache,
    DocFormatId::Cff,
    DocFormatId::GitConfig,
    DocFormatId::DockerCompose,
    DocFormatId::Cabal,
    DocFormatId::SubStationAlpha,
    DocFormatId::Sami,
    DocFormatId::ICalendar,
    DocFormatId::TeXinfo,
    DocFormatId::GeoJson,
    DocFormatId::Kml,
    DocFormatId::RssAtom,
    DocFormatId::Xslt,
    DocFormatId::Xcode,
    DocFormatId::Autoconf,
    DocFormatId::Doxygen,
    DocFormatId::Sudoers,
    DocFormatId::Passwd,
    DocFormatId::KnownHosts,
    DocFormatId::MesonWrap,
    DocFormatId::FontForge,
    DocFormatId::GitCommit,
];

impl DocFormatId {
    #[inline(always)]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Markdown => "Markdown",
            Self::PlainText => "Plain Text",
            Self::Csv => "CSV",
            Self::Svg => "SVG",
            Self::Ini => "INI / Systemd",
            Self::RestructuredText => "reStructuredText",
            Self::Patch => "Patch / Diff",
            Self::Json5 => "JSON5 / JSONC",
            Self::AsciiDoc => "AsciiDoc",
            Self::Org => "Org-mode",
            Self::BibTeX => "BibTeX",
            Self::Properties => "Properties / Env",
            Self::Gettext => "PO / Gettext",
            Self::Roff => "Roff / Man Page",
            Self::Typst => "Typst",
            Self::Djot => "Djot",
            Self::Textile => "Textile",
            Self::Gemtext => "Gemtext",
            Self::Pod => "Pod",
            Self::MediaWiki => "MediaWiki",
            Self::NdJson => "NDJSON / JSON Lines",
            Self::Avro => "Avro Schema",
            Self::Nginx => "Nginx Config",
            Self::Apache => "Apache Config",
            Self::SshConfig => "SSH Config",
            Self::Crontab => "Crontab",
            Self::GitIgnore => "Git Ignore",
            Self::Dot => "Graphviz / DOT",
            Self::EditorConfig => "EditorConfig",
            Self::Http => "HTTP Request / REST",
            Self::WebVtt => "WebVTT",
            Self::Srt => "SubRip Subtitle",
            Self::Requirements => "Pip Requirements",
            Self::PlantUML => "PlantUML",
            Self::Mermaid => "Mermaid",
            Self::Hosts => "Hosts",
            Self::Fstab => "Fstab",
            Self::Resolv => "Resolv.conf",
            Self::Log => "Log",
            Self::Xsd => "XML Schema (XSD)",
            Self::Wsdl => "WSDL",
            Self::CMakeCache => "CMake Cache",
            Self::Cff => "Citation File Format (CFF)",
            Self::GitConfig => "Git Config / Modules",
            Self::DockerCompose => "Docker Compose",
            Self::Cabal => "Cabal",
            Self::SubStationAlpha => "SubStation Alpha Subtitle",
            Self::Sami => "SAMI Subtitle",
            Self::ICalendar => "iCalendar / vCard",
            Self::TeXinfo => "TeXinfo",
            Self::GeoJson => "GeoJSON",
            Self::Kml => "Keyhole Markup Language (KML)",
            Self::RssAtom => "RSS / Atom Feed",
            Self::Xslt => "XSLT Stylesheet",
            Self::Xcode => "Xcode Project",
            Self::Autoconf => "Autoconf",
            Self::Doxygen => "Doxygen Config",
            Self::Sudoers => "Sudoers",
            Self::Passwd => "Unix Passwd / Group",
            Self::KnownHosts => "SSH Known Hosts / Keys",
            Self::MesonWrap => "Meson Wrap",
            Self::FontForge => "FontForge SFD",
            Self::GitCommit => "Git Commit / Tag Message",
        }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct FileTarget(pub u16);

impl FileTarget {
    pub const DOC_FLAG: u16 = 0x8000;

    #[inline(always)]
    pub const fn from_lang(lang_id: u8) -> Self {
        Self(lang_id as u16)
    }

    #[inline(always)]
    pub const fn from_doc(doc: DocFormatId) -> Self {
        Self((doc as u16) | Self::DOC_FLAG)
    }

    #[inline(always)]
    pub const fn is_doc(self) -> bool {
        (self.0 & Self::DOC_FLAG) != 0
    }

    #[inline(always)]
    pub const fn doc_format(self) -> Option<DocFormatId> {
        if self.is_doc() {
            let id = (self.0 & !Self::DOC_FLAG) as usize;
            if id < DOC_FORMAT_COUNT {
                return Some(ALL_DOC_FORMATS[id]);
            }
        }
        None
    }

    #[inline(always)]
    pub const fn lang_id(self) -> u8 {
        self.0 as u8
    }
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct DocStats {
    pub lines: i64,
    pub content_lines: i64,
    pub comment_lines: i64,
    pub blank_lines: i64,
}

impl DocStats {
    #[inline(always)]
    pub fn add(&mut self, other: &Self) {
        self.lines += other.lines;
        self.content_lines += other.content_lines;
        self.comment_lines += other.comment_lines;
        self.blank_lines += other.blank_lines;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AllDocStats {
    pub stats: [DocStats; DOC_FORMAT_COUNT],
}

impl Default for AllDocStats {
    #[inline(always)]
    fn default() -> Self {
        Self {
            stats: [DocStats {
                lines: 0,
                content_lines: 0,
                comment_lines: 0,
                blank_lines: 0,
            }; DOC_FORMAT_COUNT],
        }
    }
}

impl AllDocStats {
    #[inline(always)]
    pub fn add(&mut self, other: &Self) {
        let mut i = 0;
        while i < DOC_FORMAT_COUNT {
            self.stats[i].add(&other.stats[i]);
            i += 1;
        }
    }
}

#[inline(always)]
pub fn detect_doc_format(name: &[u8]) -> Option<DocFormatId> {
    if name.ends_with(b"/.git/config") || name.ends_with(b"\\.git\\config") {
        return Some(DocFormatId::GitConfig);
    }
    let name = crate::lang::get_basename(name);
    if name == b"README" || name == b"LICENSE" || name == b"COPYING" || name == b"AUTHORS" || name == b"INSTALL" {
        return Some(DocFormatId::PlainText);
    }
    if name == b".env" || name.starts_with(b".env.") {
        return Some(DocFormatId::Properties);
    }
    if name == b"nginx.conf" {
        return Some(DocFormatId::Nginx);
    }
    if name == b"httpd.conf" || name == b".htaccess" || name == b".htpasswd" {
        return Some(DocFormatId::Apache);
    }
    if name == b"ssh_config" || name == b"sshd_config" {
        return Some(DocFormatId::SshConfig);
    }
    if name == b"authorized_keys" || name == b"known_hosts" {
        return Some(DocFormatId::KnownHosts);
    }
    if name == b"Doxyfile" || name == b"Doxyfile.in" {
        return Some(DocFormatId::Doxygen);
    }
    if name == b"sudoers" {
        return Some(DocFormatId::Sudoers);
    }
    if name == b"passwd" || name == b"group" || name == b"shadow" {
        return Some(DocFormatId::Passwd);
    }
    if name == b"COMMIT_EDITMSG" || name == b"MERGE_MSG" || name == b"TAG_EDITMSG" {
        return Some(DocFormatId::GitCommit);
    }
    if name == b"configure.ac" || name == b"configure.in" {
        return Some(DocFormatId::Autoconf);
    }
    if name == b"crontab" {
        return Some(DocFormatId::Crontab);
    }
    if name == b".gitignore" || name == b".gitattributes" || name == b".gitmodules" {
        return Some(DocFormatId::GitIgnore);
    }
    if name == b".editorconfig" {
        return Some(DocFormatId::EditorConfig);
    }
    if name == b"requirements.txt" || (name.starts_with(b"requirements") && name.ends_with(b".txt")) {
        return Some(DocFormatId::Requirements);
    }
    if name == b"hosts" || name == b"hosts.allow" || name == b"hosts.deny" {
        return Some(DocFormatId::Hosts);
    }
    if name == b"fstab" {
        return Some(DocFormatId::Fstab);
    }
    if name == b"resolv.conf" {
        return Some(DocFormatId::Resolv);
    }
    if name == b"CMakeCache.txt" {
        return Some(DocFormatId::CMakeCache);
    }
    if name == b"CITATION.cff" {
        return Some(DocFormatId::Cff);
    }
    if name == b".gitconfig" || name == b".gitmodules" || name == b"gitconfig" || name.ends_with(b"/.gitconfig") || name.ends_with(b"/.gitmodules") || name.ends_with(b"/.git/config") {
        return Some(DocFormatId::GitConfig);
    }
    if name == b"docker-compose.yml" || name == b"docker-compose.yaml" || name == b"compose.yml" || name == b"compose.yaml" {
        return Some(DocFormatId::DockerCompose);
    }
    let len = name.len();
    if len < 3 {
        return None;
    }
    if len >= 3 && name[len - 2] == b'.' {
        let b = name[len - 1];
        if (b'1'..=b'8').contains(&b) {
            return Some(DocFormatId::Roff);
        }
    }
    if len >= 4 && name[len - 3] == b'.' {
        let ext = [name[len - 2] | 0x20, name[len - 1] | 0x20];
        match &ext {
            b"md" => return Some(DocFormatId::Markdown),
            b"in" => return Some(DocFormatId::PlainText),
            b"po" => return Some(DocFormatId::Gettext),
            b"dj" => return Some(DocFormatId::Djot),
            b"gv" => return Some(DocFormatId::Dot),
            b"m4" => return Some(DocFormatId::Autoconf),
            _ => {}
        }
    }
    if len >= 5 && name[len - 4] == b'.' {
        let ext = [
            name[len - 3] | 0x20,
            name[len - 2] | 0x20,
            name[len - 1] | 0x20,
        ];
        match &ext {
            b"txt" => return Some(DocFormatId::PlainText),
            b"csv" | b"tsv" => return Some(DocFormatId::Csv),
            b"svg" => return Some(DocFormatId::Svg),
            b"ini" => return Some(DocFormatId::Ini),
            b"rst" => return Some(DocFormatId::RestructuredText),
            b"org" => return Some(DocFormatId::Org),
            b"bib" => return Some(DocFormatId::BibTeX),
            b"env" => return Some(DocFormatId::Properties),
            b"pot" => return Some(DocFormatId::Gettext),
            b"man" | b"rof" => return Some(DocFormatId::Roff),
            b"typ" => return Some(DocFormatId::Typst),
            b"gmi" => return Some(DocFormatId::Gemtext),
            b"pod" => return Some(DocFormatId::Pod),
            b"dot" => return Some(DocFormatId::Dot),
            b"vtt" => return Some(DocFormatId::WebVtt),
            b"srt" => return Some(DocFormatId::Srt),
            b"pip" => return Some(DocFormatId::Requirements),
            b"mmd" => return Some(DocFormatId::Mermaid),
            b"log" => return Some(DocFormatId::Log),
            b"xsd" => return Some(DocFormatId::Xsd),
            b"cff" => return Some(DocFormatId::Cff),
            b"ssa" | b"ass" => return Some(DocFormatId::SubStationAlpha),
            b"smi" => return Some(DocFormatId::Sami),
            b"ics" | b"vcf" => return Some(DocFormatId::ICalendar),
            b"kml" => return Some(DocFormatId::Kml),
            b"rss" => return Some(DocFormatId::RssAtom),
            b"xsl" => return Some(DocFormatId::Xslt),
            b"sfd" => return Some(DocFormatId::FontForge),
            _ => {}
        }
    }
    if len >= 6 && name[len - 5] == b'.' {
        let ext = [
            name[len - 4] | 0x20,
            name[len - 3] | 0x20,
            name[len - 2] | 0x20,
            name[len - 1] | 0x20,
        ];
        match &ext {
            b"conf" => return Some(DocFormatId::Ini),
            b"diff" => return Some(DocFormatId::Patch),
            b"adoc" => return Some(DocFormatId::AsciiDoc),
            b"roff" => return Some(DocFormatId::Roff),
            b"wiki" => return Some(DocFormatId::MediaWiki),
            b"avsc" => return Some(DocFormatId::Avro),
            b"cron" => return Some(DocFormatId::Crontab),
            b"puml" => return Some(DocFormatId::PlantUML),
            b"wsdl" => return Some(DocFormatId::Wsdl),
            b"http" | b"rest" => return Some(DocFormatId::Http),
            b"sami" => return Some(DocFormatId::Sami),
            b"texi" => return Some(DocFormatId::TeXinfo),
            b"atom" => return Some(DocFormatId::RssAtom),
            b"xslt" => return Some(DocFormatId::Xslt),
            b"wrap" => return Some(DocFormatId::MesonWrap),
            _ => {}
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
        match &ext {
            b"patch" => return Some(DocFormatId::Patch),
            b"json5" | b"jsonc" => return Some(DocFormatId::Json5),
            b"jsonl" => return Some(DocFormatId::NdJson),
            b"nginx" => return Some(DocFormatId::Nginx),
            b"cabal" => return Some(DocFormatId::Cabal),
            b"vcard" => return Some(DocFormatId::ICalendar),
            _ => {}
        }
    }
    if len >= 8 && name[len - 7] == b'.' {
        let ext = [
            name[len - 6] | 0x20,
            name[len - 5] | 0x20,
            name[len - 4] | 0x20,
            name[len - 3] | 0x20,
            name[len - 2] | 0x20,
            name[len - 1] | 0x20,
        ];
        match &ext {
            b"socket" => return Some(DocFormatId::Ini),
            b"ndjson" => return Some(DocFormatId::NdJson),
            _ => {}
        }
    }
    if len >= 9 && name[len - 8] == b'.' {
        let ext = [
            name[len - 7] | 0x20,
            name[len - 6] | 0x20,
            name[len - 5] | 0x20,
            name[len - 4] | 0x20,
            name[len - 3] | 0x20,
            name[len - 2] | 0x20,
            name[len - 1] | 0x20,
        ];
        match &ext {
            b"service" => return Some(DocFormatId::Ini),
            b"textile" => return Some(DocFormatId::Textile),
            b"mermaid" => return Some(DocFormatId::Mermaid),
            b"texinfo" => return Some(DocFormatId::TeXinfo),
            b"geojson" => return Some(DocFormatId::GeoJson),
            b"pbxproj" => return Some(DocFormatId::Xcode),
            _ => {}
        }
    }
    if len >= 10 && name[len - 9] == b'.' {
        let ext = [
            name[len - 8] | 0x20,
            name[len - 7] | 0x20,
            name[len - 6] | 0x20,
            name[len - 5] | 0x20,
            name[len - 4] | 0x20,
            name[len - 3] | 0x20,
            name[len - 2] | 0x20,
            name[len - 1] | 0x20,
        ];
        if &ext == b"plantuml" {
            return Some(DocFormatId::PlantUML);
        }
    }
    if len >= 10 && name[len - 9] == b'.' {
        let ext = [
            name[len - 8] | 0x20,
            name[len - 7] | 0x20,
            name[len - 6] | 0x20,
            name[len - 5] | 0x20,
            name[len - 4] | 0x20,
            name[len - 3] | 0x20,
            name[len - 2] | 0x20,
            name[len - 1] | 0x20,
        ];
        if &ext == b"asciidoc" {
            return Some(DocFormatId::AsciiDoc);
        }
    }
    if len >= 11 && name[len - 10] == b'.' {
        let ext = [
            name[len - 9] | 0x20,
            name[len - 8] | 0x20,
            name[len - 7] | 0x20,
            name[len - 6] | 0x20,
            name[len - 5] | 0x20,
            name[len - 4] | 0x20,
            name[len - 3] | 0x20,
            name[len - 2] | 0x20,
            name[len - 1] | 0x20,
        ];
        if &ext == b"mediawiki" {
            return Some(DocFormatId::MediaWiki);
        }
    }
    if len >= 12 && name[len - 11] == b'.' {
        let ext = [
            name[len - 10] | 0x20,
            name[len - 9] | 0x20,
            name[len - 8] | 0x20,
            name[len - 7] | 0x20,
            name[len - 6] | 0x20,
            name[len - 5] | 0x20,
            name[len - 4] | 0x20,
            name[len - 3] | 0x20,
            name[len - 2] | 0x20,
            name[len - 1] | 0x20,
        ];
        match &ext {
            b"properties" => return Some(DocFormatId::Properties),
            b"apacheconf" => return Some(DocFormatId::Apache),
            _ => {}
        }
    }
    None
}

fn count_block_comment_doc_line(
    trimmed: &[u8],
    open: &[u8],
    close: &[u8],
    in_block: &mut bool,
    stats: &mut DocStats,
) {
    let mut has_content = false;
    let mut has_comment = *in_block;
    let mut rest = trimmed;
    while !rest.is_empty() {
        if *in_block {
            if let Some(close_pos) = rest.windows(close.len()).position(|w| w == close) {
                *in_block = false;
                rest = &rest[close_pos + close.len()..];
            } else {
                break;
            }
        } else {
            let mut ws = 0;
            while ws < rest.len() && (rest[ws] == b' ' || rest[ws] == b'\t') {
                ws += 1;
            }
            rest = &rest[ws..];
            if rest.is_empty() {
                break;
            }
            if rest.starts_with(open) {
                has_comment = true;
                *in_block = true;
                rest = &rest[open.len()..];
            } else {
                has_content = true;
                if let Some(open_pos) = rest.windows(open.len()).position(|w| w == open) {
                    has_comment = true;
                    *in_block = true;
                    rest = &rest[open_pos + open.len()..];
                } else {
                    break;
                }
            }
        }
    }
    if has_content {
        stats.content_lines += 1;
    } else if has_comment {
        stats.comment_lines += 1;
    } else {
        stats.blank_lines += 1;
    }
}

fn count_block_comment_doc_line_caseless(
    trimmed: &[u8],
    open: &[u8],
    close: &[u8],
    in_block: &mut bool,
    stats: &mut DocStats,
) {
    let mut has_content = false;
    let mut has_comment = *in_block;
    let mut rest = trimmed;
    while !rest.is_empty() {
        if *in_block {
            if let Some(close_pos) = rest.windows(close.len()).position(|w| w.eq_ignore_ascii_case(close)) {
                *in_block = false;
                rest = &rest[close_pos + close.len()..];
            } else {
                break;
            }
        } else {
            let mut ws = 0;
            while ws < rest.len() && (rest[ws] == b' ' || rest[ws] == b'\t') {
                ws += 1;
            }
            rest = &rest[ws..];
            if rest.is_empty() {
                break;
            }
            if rest.len() >= open.len() && rest[..open.len()].eq_ignore_ascii_case(open) {
                has_comment = true;
                *in_block = true;
                rest = &rest[open.len()..];
            } else {
                has_content = true;
                if let Some(open_pos) = rest.windows(open.len()).position(|w| w.eq_ignore_ascii_case(open)) {
                    has_comment = true;
                    *in_block = true;
                    rest = &rest[open_pos + open.len()..];
                } else {
                    break;
                }
            }
        }
    }
    if has_content {
        stats.content_lines += 1;
    } else if has_comment {
        stats.comment_lines += 1;
    } else {
        stats.blank_lines += 1;
    }
}

fn count_org_doc_line(trimmed: &[u8], in_block: &mut bool, stats: &mut DocStats) {
    if !*in_block && (trimmed.starts_with(b"# ") || trimmed.starts_with(b"#\t") || trimmed == b"#") {
        stats.comment_lines += 1;
        return;
    }
    count_block_comment_doc_line_caseless(trimmed, b"#+BEGIN_COMMENT", b"#+END_COMMENT", in_block, stats);
}

fn count_json5_doc_line(trimmed: &[u8], in_block: &mut bool, stats: &mut DocStats) {
    let mut has_content = false;
    let mut has_comment = *in_block;
    let mut idx = 0usize;
    let len = trimmed.len();
    while idx < len {
        if *in_block {
            if idx + 1 < len && trimmed[idx] == b'*' && trimmed[idx + 1] == b'/' {
                *in_block = false;
                idx += 2;
            } else {
                idx += 1;
            }
            continue;
        }
        let b = trimmed[idx];
        if b == b' ' || b == b'\t' {
            idx += 1;
            continue;
        }
        if idx + 1 < len && b == b'/' && trimmed[idx + 1] == b'/' {
            has_comment = true;
            break;
        }
        if idx + 1 < len && b == b'/' && trimmed[idx + 1] == b'*' {
            has_comment = true;
            *in_block = true;
            idx += 2;
            continue;
        }
        has_content = true;
        if b == b'"' || b == b'\'' {
            let quote = b;
            idx += 1;
            while idx < len {
                if trimmed[idx] == b'\\' {
                    idx += 2;
                } else if trimmed[idx] == quote {
                    idx += 1;
                    break;
                } else {
                    idx += 1;
                }
            }
        } else {
            idx += 1;
        }
    }
    if has_content {
        stats.content_lines += 1;
    } else if has_comment {
        stats.comment_lines += 1;
    } else {
        stats.blank_lines += 1;
    }
}

fn is_rst_comment(trimmed: &[u8]) -> bool {
    if trimmed == b".." {
        return true;
    }
    let Some(rest) = trimmed.strip_prefix(b".. ") else {
        return false;
    };
    let mut ws = 0;
    while ws < rest.len() && (rest[ws] == b' ' || rest[ws] == b'\t') {
        ws += 1;
    }
    let rest = &rest[ws..];
    if rest.is_empty() {
        return true;
    }
    if matches!(rest[0], b'_' | b'[' | b'|') {
        return false;
    }
    let mut p = 0;
    while p < rest.len() {
        let b = rest[p];
        if b.is_ascii_alphanumeric()
            || matches!(b, b'-' | b'_' | b'.' | b'+')
            || (b == b':' && p + 1 < rest.len() && rest[p + 1] != b':')
        {
            p += 1;
        } else {
            break;
        }
    }
    if p > 0
        && rest[p..].starts_with(b"::")
        && (p + 2 == rest.len() || rest[p + 2] == b' ' || rest[p + 2] == b'\t')
    {
        return false;
    }
    true
}

fn count_asciidoc_doc_line(trimmed: &[u8], in_block: &mut bool, stats: &mut DocStats) {
    if !*in_block && trimmed.starts_with(b"//") && !trimmed.starts_with(b"////") {
        stats.comment_lines += 1;
        return;
    }
    count_block_comment_doc_line(trimmed, b"////", b"////", in_block, stats);
}

fn count_bibtex_doc_line(trimmed: &[u8], stats: &mut DocStats) {
    if trimmed.starts_with(b"%") {
        stats.comment_lines += 1;
        return;
    }
    if trimmed.len() >= 8 && trimmed[..8].eq_ignore_ascii_case(b"@comment") {
        stats.comment_lines += 1;
        return;
    }
    stats.content_lines += 1;
}

pub fn parse_doc_into(doc: DocFormatId, content: &[u8], stats: &mut DocStats) {
    if content.is_empty() {
        return;
    }
    let mut i = 0usize;
    let n = content.len();
    let mut in_block = false;

    while i < n {
        let line_start = i;
        while i < n && content[i] != b'\n' {
            i += 1;
        }
        let mut line_end = i;
        if i < n && content[i] == b'\n' {
            i += 1;
        }
        if line_end > line_start && content[line_end - 1] == b'\r' {
            line_end -= 1;
        }
        let line = &content[line_start..line_end];
        stats.lines += 1;

        let mut trimmed_start = 0;
        while trimmed_start < line.len() && (line[trimmed_start] == b' ' || line[trimmed_start] == b'\t') {
            trimmed_start += 1;
        }
        if trimmed_start == line.len() {
            if in_block {
                stats.comment_lines += 1;
            } else {
                stats.blank_lines += 1;
            }
            continue;
        }
        let trimmed = &line[trimmed_start..];

        match doc {
            DocFormatId::Markdown | DocFormatId::Svg => {
                count_block_comment_doc_line(trimmed, b"<!--", b"-->", &mut in_block, stats);
            }
            DocFormatId::PlainText | DocFormatId::Csv => {
                stats.content_lines += 1;
            }
            DocFormatId::Ini => {
                if trimmed.starts_with(b"#") || trimmed.starts_with(b";") {
                    stats.comment_lines += 1;
                } else {
                    stats.content_lines += 1;
                }
            }
            DocFormatId::RestructuredText => {
                if is_rst_comment(trimmed) {
                    stats.comment_lines += 1;
                } else {
                    stats.content_lines += 1;
                }
            }
            DocFormatId::Patch => {
                stats.content_lines += 1;
            }
            DocFormatId::Json5 => {
                count_json5_doc_line(trimmed, &mut in_block, stats);
            }
            DocFormatId::AsciiDoc => {
                count_asciidoc_doc_line(trimmed, &mut in_block, stats);
            }
            DocFormatId::Org => {
                count_org_doc_line(trimmed, &mut in_block, stats);
            }
            DocFormatId::BibTeX => {
                count_bibtex_doc_line(trimmed, stats);
            }
            DocFormatId::Properties => {
                if trimmed.starts_with(b"#") || trimmed.starts_with(b"!") {
                    stats.comment_lines += 1;
                } else {
                    stats.content_lines += 1;
                }
            }
            DocFormatId::Gettext => {
                if trimmed.starts_with(b"#") {
                    stats.comment_lines += 1;
                } else {
                    stats.content_lines += 1;
                }
            }
            DocFormatId::Roff => {
                if trimmed.starts_with(b".\\\"") || trimmed.starts_with(b"'\\\"") {
                    stats.comment_lines += 1;
                } else {
                    stats.content_lines += 1;
                }
            }
            DocFormatId::Typst => {
                if !in_block && trimmed.starts_with(b"//") {
                    stats.comment_lines += 1;
                } else {
                    count_block_comment_doc_line(trimmed, b"/*", b"*/", &mut in_block, stats);
                }
            }
            DocFormatId::Djot => {
                count_block_comment_doc_line(trimmed, b"{%", b"%}", &mut in_block, stats);
            }
            DocFormatId::Textile => {
                if trimmed.starts_with(b"###.") {
                    stats.comment_lines += 1;
                } else {
                    stats.content_lines += 1;
                }
            }
            DocFormatId::Gemtext => {
                stats.content_lines += 1;
            }
            DocFormatId::Pod => {
                count_block_comment_doc_line(trimmed, b"=begin comment", b"=end comment", &mut in_block, stats);
            }
            DocFormatId::MediaWiki => {
                count_block_comment_doc_line(trimmed, b"<!--", b"-->", &mut in_block, stats);
            }
            DocFormatId::NdJson | DocFormatId::Avro => {
                stats.content_lines += 1;
            }
            DocFormatId::Nginx
            | DocFormatId::Apache
            | DocFormatId::SshConfig
            | DocFormatId::Crontab
            | DocFormatId::GitIgnore => {
                if trimmed.starts_with(b"#") {
                    stats.comment_lines += 1;
                } else {
                    stats.content_lines += 1;
                }
            }
            DocFormatId::Dot => {
                if !in_block && (trimmed.starts_with(b"//") || trimmed.starts_with(b"#")) {
                    stats.comment_lines += 1;
                } else {
                    count_block_comment_doc_line(trimmed, b"/*", b"*/", &mut in_block, stats);
                }
            }
            DocFormatId::EditorConfig => {
                if trimmed.starts_with(b"#") || trimmed.starts_with(b";") {
                    stats.comment_lines += 1;
                } else {
                    stats.content_lines += 1;
                }
            }
            DocFormatId::Http => {
                if trimmed.starts_with(b"#") || trimmed.starts_with(b"//") {
                    stats.comment_lines += 1;
                } else {
                    stats.content_lines += 1;
                }
            }
            DocFormatId::WebVtt => {
                if trimmed.starts_with(b"NOTE") && (trimmed.len() == 4 || trimmed[4] == b' ' || trimmed[4] == b'\t') {
                    stats.comment_lines += 1;
                } else {
                    stats.content_lines += 1;
                }
            }
            DocFormatId::Srt => {
                stats.content_lines += 1;
            }
            DocFormatId::Requirements => {
                if trimmed.starts_with(b"#") {
                    stats.comment_lines += 1;
                } else {
                    stats.content_lines += 1;
                }
            }
            DocFormatId::PlantUML => {
                if !in_block && trimmed.starts_with(b"'") {
                    stats.comment_lines += 1;
                } else {
                    count_block_comment_doc_line(trimmed, b"/'", b"'/", &mut in_block, stats);
                }
            }
            DocFormatId::Mermaid => {
                if trimmed.starts_with(b"%%") {
                    stats.comment_lines += 1;
                } else {
                    stats.content_lines += 1;
                }
            }
            DocFormatId::Hosts | DocFormatId::Fstab | DocFormatId::Cff => {
                if trimmed.starts_with(b"#") {
                    stats.comment_lines += 1;
                } else {
                    stats.content_lines += 1;
                }
            }
            DocFormatId::Resolv => {
                if trimmed.starts_with(b"#") || trimmed.starts_with(b";") {
                    stats.comment_lines += 1;
                } else {
                    stats.content_lines += 1;
                }
            }
            DocFormatId::Log => {
                stats.content_lines += 1;
            }
            DocFormatId::Xsd | DocFormatId::Wsdl => {
                count_block_comment_doc_line(trimmed, b"<!--", b"-->", &mut in_block, stats);
            }
            DocFormatId::GitConfig => {
                if trimmed.starts_with(b"#") || trimmed.starts_with(b";") {
                    stats.comment_lines += 1;
                } else {
                    stats.content_lines += 1;
                }
            }
            DocFormatId::DockerCompose => {
                if trimmed.starts_with(b"#") {
                    stats.comment_lines += 1;
                } else {
                    stats.content_lines += 1;
                }
            }
            DocFormatId::Cabal => {
                if trimmed.starts_with(b"--") {
                    stats.comment_lines += 1;
                } else {
                    stats.content_lines += 1;
                }
            }
            DocFormatId::SubStationAlpha => {
                if trimmed.starts_with(b";") {
                    stats.comment_lines += 1;
                } else {
                    stats.content_lines += 1;
                }
            }
            DocFormatId::Sami | DocFormatId::Kml | DocFormatId::RssAtom => {
                count_block_comment_doc_line(trimmed, b"<!--", b"-->", &mut in_block, stats);
            }
            DocFormatId::ICalendar | DocFormatId::GeoJson => {
                stats.content_lines += 1;
            }
            DocFormatId::TeXinfo => {
                if trimmed.starts_with(b"@c ") || trimmed.starts_with(b"@comment ") || trimmed == b"@c" || trimmed == b"@comment" {
                    stats.comment_lines += 1;
                } else {
                    stats.content_lines += 1;
                }
            }
            DocFormatId::CMakeCache => {
                if trimmed.starts_with(b"#") || trimmed.starts_with(b"//") {
                    stats.comment_lines += 1;
                } else {
                    stats.content_lines += 1;
                }
            }
            DocFormatId::Xslt => {
                count_block_comment_doc_line(trimmed, b"<!--", b"-->", &mut in_block, stats);
            }
            DocFormatId::Xcode => {
                if !in_block && trimmed.starts_with(b"//") {
                    stats.comment_lines += 1;
                } else {
                    count_block_comment_doc_line(trimmed, b"/*", b"*/", &mut in_block, stats);
                }
            }
            DocFormatId::Autoconf => {
                if trimmed.starts_with(b"#") || trimmed.starts_with(b"dnl ") || trimmed == b"dnl" {
                    stats.comment_lines += 1;
                } else {
                    stats.content_lines += 1;
                }
            }
            DocFormatId::Doxygen
            | DocFormatId::Sudoers
            | DocFormatId::Passwd
            | DocFormatId::KnownHosts
            | DocFormatId::MesonWrap
            | DocFormatId::FontForge
            | DocFormatId::GitCommit => {
                if trimmed.starts_with(b"#") {
                    stats.comment_lines += 1;
                } else {
                    stats.content_lines += 1;
                }
            }
        }
    }
}
