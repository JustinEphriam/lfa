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
}

pub const DOC_FORMAT_COUNT: usize = 8;
pub const ALL_DOC_FORMATS: [DocFormatId; DOC_FORMAT_COUNT] = [
    DocFormatId::Markdown,
    DocFormatId::PlainText,
    DocFormatId::Csv,
    DocFormatId::Svg,
    DocFormatId::Ini,
    DocFormatId::RestructuredText,
    DocFormatId::Patch,
    DocFormatId::Json5,
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
        }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct FileTarget(pub u8);

impl FileTarget {
    pub const DOC_FLAG: u8 = 0x80;

    #[inline(always)]
    pub const fn from_lang(lang_id: u8) -> Self {
        Self(lang_id)
    }

    #[inline(always)]
    pub const fn from_doc(doc: DocFormatId) -> Self {
        Self((doc as u8) | Self::DOC_FLAG)
    }

    #[inline(always)]
    pub const fn is_doc(self) -> bool {
        (self.0 & Self::DOC_FLAG) != 0
    }

    #[inline(always)]
    pub const fn doc_format(self) -> Option<DocFormatId> {
        if self.is_doc() {
            let id = self.0 & !Self::DOC_FLAG;
            if (id as usize) < DOC_FORMAT_COUNT {
                return Some(ALL_DOC_FORMATS[id as usize]);
            }
        }
        None
    }

    #[inline(always)]
    pub const fn lang_id(self) -> u8 {
        self.0
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

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct AllDocStats {
    pub stats: [DocStats; DOC_FORMAT_COUNT],
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
    if name == b"README" || name == b"LICENSE" || name == b"COPYING" || name == b"AUTHORS" || name == b"INSTALL" {
        return Some(DocFormatId::PlainText);
    }
    let len = name.len();
    if len < 4 {
        return None;
    }
    if len >= 4 && name[len - 3] == b'.' {
        let ext = [name[len - 2] | 0x20, name[len - 1] | 0x20];
        match &ext {
            b"md" => return Some(DocFormatId::Markdown),
            b"in" => return Some(DocFormatId::PlainText),
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
            b"txt" | b"log" => return Some(DocFormatId::PlainText),
            b"csv" | b"tsv" => return Some(DocFormatId::Csv),
            b"svg" => return Some(DocFormatId::Svg),
            b"ini" => return Some(DocFormatId::Ini),
            b"rst" => return Some(DocFormatId::RestructuredText),
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
        if &ext == b"socket" {
            return Some(DocFormatId::Ini);
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
        if &ext == b"service" {
            return Some(DocFormatId::Ini);
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
            stats.blank_lines += 1;
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
        }
    }
}
