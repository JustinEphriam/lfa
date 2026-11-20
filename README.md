# lfa

`lfa` is a command-line tool that scans a file or directory and reports, per language, the number of files, lines, code lines, comment lines, blank lines, and language keywords. It runs on multiple threads and recognizes 192 programming languages plus a set of document and data formats.

## Build

Requires a Rust toolchain with edition 2024 support.

```bash
cargo build --release
```

The binary is written to `target/release/lfa`.

## Usage

```
lfa [OPTIONS] <PATH>
```

`<PATH>` is a file or a directory. Directories are searched recursively.

```bash
lfa .
lfa src/main.rs
lfa -rust -cpp /path/to/code
```

## What it reports

For each detected language:

| Column   | Meaning                                  |
|:---------|:-----------------------------------------|
| Files    | Number of files of that language         |
| Lines    | Total lines                              |
| Code     | Lines containing code                    |
| Comments | Lines that are comments                  |
| Blanks   | Empty lines                              |
| Keywords | Number of language keywords encountered  |

Lines always equal Code + Comments + Blanks.

Document and data formats (Markdown, plain text, CSV, INI, and others) are reported in a separate table with the columns Files, Lines, Content, Comments, and Blanks. Keywords are not counted for these.

The run ends with a line showing the number of files processed, the number of worker threads, and the elapsed time.

Example:

```
┌────────────────────┬──────────┬──────────────┬──────────────┬──────────────┬──────────────┬──────────────┐
│ Language           │    Files │        Lines │         Code │     Comments │       Blanks │     Keywords │
├────────────────────┼──────────┼──────────────┼──────────────┼──────────────┼──────────────┼──────────────┤
│ Rust               │      194 │       53,687 │       49,342 │            0 │        4,345 │       26,737 │
│ TOML               │        1 │           18 │           15 │            0 │            3 │            1 │
│ Dockerfile         │        1 │          129 │          114 │            4 │           11 │            0 │
├────────────────────┼──────────┼──────────────┼──────────────┼──────────────┼──────────────┼──────────────┤
│ Total              │      196 │       53,834 │       49,471 │            4 │        4,359 │       26,738 │
└────────────────────┴──────────┴──────────────┴──────────────┴──────────────┴──────────────┴──────────────┘
```

## Options

| Option                          | Description                                                          |
|:--------------------------------|:---------------------------------------------------------------------|
| `-h`, `--help`                  | Print help and the list of language flags                            |
| `-V`, `--version`               | Print the version                                                    |
| `-f`, `--format <FORMAT>`       | Output format: `table` (default), `json`, `csv`, `markdown` or `md`  |
| `--json`, `--csv`, `--markdown`, `--md` | Shorthand for `--format`                                     |
| `--hidden`                      | Include hidden files and directories (names starting with `.`)       |
| `--sort <COLUMN>`               | Sort the summary table by a column                                   |
| `--top <N>`                     | Show only the first `N` languages and combine the rest into `Other`  |
| Language flags                  | Restrict the scan to specific languages and show keyword frequencies |

Options that take a value also accept the `--option=value` form, for example `--format=csv`, `--sort=lines`, `--top=5`.

### Hidden files

Files and directories whose names start with `.` are skipped by default. Pass `--hidden` to include them.

### Sorting

The summary table is sorted by keyword count, highest first, unless another column is chosen. Ties are broken by language name.

Valid columns for `--sort`: `keywords` (`kws`), `lines` (`line`), `code`, `comments` (`comment`, `comm`), `blanks` (`blank`), `files` (`file`), `name` (`lang`).

Each column also has a dedicated flag: `--sort-keywords`, `--sort-lines`, `--sort-code`, `--sort-comments`, `--sort-blanks`, `--sort-files`, `--sort-name`.

```bash
lfa . --sort lines
lfa . --sort-code
```

### Limiting rows

`--top N` keeps the first `N` rows in the current sort order and sums the remaining languages into one `Other` row. Totals are unchanged.

```bash
lfa . --top 3 --sort lines
```

### Output formats

```bash
lfa . --json
lfa . --csv
lfa . --md
lfa . -f json
```

- **table**: box-drawn tables for terminals. Color is applied when the output supports it.
- **json**: a `summary` object with `languages` and `total`, and a `meta` object with file count, worker count, CPU core count, and processing time in seconds.
- **csv**: one header row followed by one row per language and a `Total` row.
- **markdown**: a Markdown table.

## Language filters

Passing one or more language flags limits the scan to those languages. For each selected language, `lfa` also prints a keyword frequency table with rank, keyword, count, and percent of all keywords in that language.

```bash
lfa -rust .
lfa -rust -cpp /path/to/code
lfa --python --json src
```

Each language accepts several spellings with either one or two dashes, for example `-rs`, `--rs`, `-rust`, `--rust`. Selecting `-c` also includes C headers, and `-cpp` also includes C++ headers. Run `lfa --help` for the complete list of languages and their flags.

When a language filter is given, document and data formats are not counted.

### Keyword frequency example

```
lfa -rust src/main.rs
```

prints the Rust summary row followed by a table such as:

```
│   Rank │ Keyword            │        Count │   Percent │
│      1 │ let                │          666 │    38.90% │
│      2 │ mut                │          252 │    14.72% │
│      3 │ if                 │          167 │     9.75% │
```

## File detection

- Languages are identified by file name and extension.
- Files with no extension are checked for a shebang line (for example `#!/usr/bin/env python3`).
- Document and data formats are identified by file name or extension (for example `.md`, `.txt`, `.csv`, `.ini`, `.rst`, `.patch`, `.svg`, `.log`, `.gitignore`, `crontab`, `nginx.conf`).
- Only regular files are read. Symbolic links are not followed.

## Exit status

| Code | Meaning                                         |
|:-----|:------------------------------------------------|
| 0    | Success, or help/version was printed            |
| 2    | Invalid arguments (unknown flag, missing path)  |

Errors are written to standard error.

## Development

```bash
cargo test
cargo clippy --all-targets
cargo build --release
```

[flow.md](flow.md) describes the process for adding a language or document format and for fixing lexer bugs.

## License

See [LICENSE](LICENSE).
