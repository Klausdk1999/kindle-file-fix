# Kindle File Fix — Design Document

**Date:** 2026-02-22
**Status:** Approved
**Repo:** https://github.com/Klausdk1999/kindle-file-fix.git
**License:** Apache 2.0

## Problem

Amazon's Send-to-Kindle service has several compatibility issues with EPUB files:
- Assumes ISO-8859-1 encoding when no XML encoding declaration is present
- Rejects hyperlinks that reference `<body>` element IDs
- Requires valid language metadata
- Fails on `<img>` tags with no `src` attribute

The original [kindle-epub-fix](https://kindle-epub-fix.netlify.app) solves this as a browser-based web app. This project reimplements it as a cross-platform CLI tool and desktop GUI application in Rust.

## Goals

1. Port all 4 EPUB fixes from the original JavaScript to Rust
2. Provide a CLI tool that produces small, static binaries for Windows/macOS/Linux
3. Provide a Tauri-based GUI app with drag-and-drop for non-technical users
4. Extensible architecture supporting future MOBI and AZW3 formats
5. Full type safety, error handling, and test coverage

## Architecture

Pure Rust monorepo using a Cargo workspace with 3 crates:

```
kindle-file-fix/
├── Cargo.toml              # Workspace root
├── crates/
│   ├── kindle-fix-core/    # Library: all fixing logic
│   ├── kindle-fix-cli/     # Binary: CLI tool
│   └── kindle-fix-gui/     # Tauri app: desktop GUI
├── gui/                    # Frontend assets for Tauri (vanilla TS + HTML/CSS)
├── tests/
│   └── fixtures/           # Sample EPUB/MOBI/AZW3 files for testing
├── docs/
│   └── plans/
├── .github/
│   └── workflows/          # CI + release workflows
├── LICENSE                 # Apache 2.0
└── README.md
```

## Core Library (`kindle-fix-core`)

### Module Structure

```
crates/kindle-fix-core/src/
├── lib.rs              # Public API: process_file(), FixResult
├── formats/
│   ├── mod.rs          # FileFixer trait + format detection
│   ├── epub/
│   │   ├── mod.rs      # EpubFixer implementation
│   │   ├── reader.rs   # EPUB ZIP reading (text vs binary separation)
│   │   ├── writer.rs   # EPUB ZIP writing (mimetype handling)
│   │   └── fixes/
│   │       ├── mod.rs
│   │       ├── encoding.rs     # UTF-8 XML declaration fix
│   │       ├── body_id.rs      # Body ID link reference fix
│   │       ├── language.rs     # Language tag validation/fix
│   │       └── stray_img.rs    # Stray <img> removal
│   ├── mobi/
│   │   └── mod.rs      # MobiFixer (stub, future implementation)
│   └── azw3/
│       └── mod.rs      # Azw3Fixer (stub, future implementation)
├── types.rs            # Shared types
└── error.rs            # Error types (thiserror)
```

### Key Trait

```rust
pub trait FileFixer {
    /// Detect if the given data matches this format
    fn detect(data: &[u8]) -> bool;

    /// Apply all fixes and return the fixed data with a report
    fn fix(&self, data: &[u8], options: &FixOptions) -> Result<FixOutput>;
}
```

### Key Types

```rust
pub struct FixOptions {
    pub language: Option<String>,    // Override language (skip prompt)
    pub keep_name: bool,             // Don't add prefix to filename
    pub dry_run: bool,               // Report only, don't write
}

pub struct FixReport {
    pub filename: String,
    pub format: FileFormat,
    pub fixes_applied: Vec<FixDescription>,
    pub warnings: Vec<String>,
}

pub struct FixDescription {
    pub name: String,           // e.g., "encoding", "body_id"
    pub details: String,        // e.g., "Added UTF-8 encoding to 12 files"
    pub files_affected: usize,
}

pub struct FixOutput {
    pub data: Vec<u8>,
    pub report: FixReport,
}

pub enum FileFormat {
    Epub,
    Mobi,
    Azw3,
    Unknown,
}
```

### Rust Dependencies (core)

| Crate | Purpose |
|-------|---------|
| `zip` | Read/write ZIP archives (EPUB is ZIP) |
| `quick-xml` | Fast XML parsing for OPF, NCX, XHTML |
| `regex` | Encoding declaration detection |
| `thiserror` | Ergonomic error types |
| `log` | Logging facade |

### EPUB Fix Details

**1. Encoding Fix (`encoding.rs`)**
- Scan all `.html`, `.xhtml`, `.htm` files in the EPUB
- Check for existing XML declaration with encoding attribute using regex:
  `^<\?xml\s+version=["'][\d.]+["']\s+encoding=["'][a-zA-Z\d-.]+["'].*?\?>`
- If missing, prepend `<?xml version="1.0" encoding="utf-8"?>`

**2. Body ID Link Fix (`body_id.rs`)**
- First pass: scan all HTML/XHTML for `<body id="...">` elements, collect IDs
- Second pass: in NCX and all content files, replace `filename.html#bodyid` with `filename.html`
- Only strip hash when the referenced ID belongs to a `<body>` element

**3. Language Tag Fix (`language.rs`)**
- Locate OPF file via `META-INF/container.xml`
- Parse OPF XML, find `<dc:language>` element
- Validate against list of 45+ Amazon-supported language codes
- If missing or invalid: use provided `--language` option or return a warning
- Supported codes: ISO 639-1 (en, fr, ja...) and ISO 639-2 (eng, fra, jpn...)

**4. Stray Image Fix (`stray_img.rs`)**
- Parse each HTML/XHTML file
- Find `<img>` elements without `src` attribute
- Remove them from the DOM
- Reserialize the modified content

### EPUB Reader/Writer

**Reader (`reader.rs`):**
- Open EPUB as ZIP archive
- Classify entries as text (html, xhtml, xml, svg, css, opf, ncx, mimetype) or binary
- Read text entries as UTF-8 strings, binary entries as `Vec<u8>`

**Writer (`writer.rs`):**
- Write `mimetype` entry FIRST with no compression (EPUB spec requirement)
- Write all other entries with standard deflate compression
- No extended timestamps on mimetype entry

## CLI Tool (`kindle-fix-cli`)

### Module Structure

```
crates/kindle-fix-cli/src/
├── main.rs             # Entry point, clap argument parsing
└── output.rs           # Terminal output formatting
```

### CLI Interface

```
kindle-file-fix [OPTIONS] <FILES>...

Arguments:
  <FILES>...          Input files or directories to process

Options:
  -o, --output <DIR>  Output directory (default: same as input)
  -l, --language <LANG>  Set language code (skip interactive prompt)
  --keep-name         Keep original filename (no prefix)
  --dry-run           Show fixes without writing files
  -v, --verbose       Verbose output
  -q, --quiet         Suppress output except errors
  -h, --help          Print help
  -V, --version       Print version
```

### CLI Dependencies

| Crate | Purpose |
|-------|---------|
| `clap` (derive) | Argument parsing |
| `colored` | Terminal colors |
| `indicatif` | Progress bars for batch processing |
| `dialoguer` | Interactive prompts (language selection) |
| `env_logger` | Logging implementation |

### Output Format

```
Processing: book.epub
  [FIXED] Added UTF-8 encoding to 12 files
  [FIXED] Removed 3 body ID link references
  [OK]    Language tag: en
  [OK]    No stray images found
  Saved: (fixed) book.epub

Processed 1 file, 2 fixes applied.
```

## GUI Application (`kindle-fix-gui`)

### Structure

```
crates/kindle-fix-gui/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs       # Tauri entry point
│   │   └── commands.rs   # Tauri IPC commands
│   ├── Cargo.toml
│   └── tauri.conf.json
gui/
├── index.html
├── main.ts
├── styles.css
└── tsconfig.json
```

### Tauri Commands

```rust
#[tauri::command]
fn process_files(paths: Vec<String>, options: FixOptions) -> Result<Vec<FixReport>, String>;

#[tauri::command]
fn get_supported_languages() -> Vec<Language>;
```

### Frontend (Vanilla TypeScript + HTML/CSS)

- Drag-and-drop zone or file picker button
- Accepts `.epub` (and future `.mobi`, `.azw3`) files
- Processing progress per file
- Fix report display (green for fixes, blue for clean, red for errors)
- Save button per file + "Save All" for batch
- Dark/light theme following system preference
- Window size: ~600x500px

## CI/CD

### `ci.yml` — On every PR and push to main

1. `cargo fmt --check`
2. `cargo clippy -- -D warnings`
3. `cargo test`
4. Build CLI for: linux-x64, macos-x64, macos-arm64, windows-x64

### `release.yml` — On tag push (`v*`)

1. Build CLI static binaries (4 targets)
2. Build Tauri installers:
   - Windows: `.msi`
   - macOS: `.dmg`
   - Linux: `.AppImage` + `.deb`
3. Create GitHub Release with all artifacts
4. Generate changelog from commit messages

### Release Artifacts

| Artifact | Size (est.) |
|----------|-------------|
| `kindle-file-fix-cli-linux-x64` | ~5MB |
| `kindle-file-fix-cli-macos-x64` | ~5MB |
| `kindle-file-fix-cli-macos-arm64` | ~5MB |
| `kindle-file-fix-cli-windows-x64.exe` | ~5MB |
| `kindle-file-fix-gui-windows.msi` | ~10-15MB |
| `kindle-file-fix-gui-macos.dmg` | ~10-15MB |
| `kindle-file-fix-gui-linux.AppImage` | ~10-15MB |

## Testing Strategy

### Unit Tests (in `kindle-fix-core`)
- Each fix function tested independently
- Test with known-broken EPUB content
- Test edge cases: empty files, missing elements, multiple issues

### Integration Tests
- `tests/fixtures/` directory with sample EPUB files:
  - `missing-encoding.epub` — no XML encoding declaration
  - `body-id-links.epub` — body ID references in NCX
  - `missing-language.epub` — no dc:language in OPF
  - `stray-images.epub` — img tags without src
  - `all-issues.epub` — all 4 problems combined
  - `clean.epub` — no issues (should pass through unchanged)
- Verify fixed output against expected results

### CLI Integration Tests
- Run binary with test fixtures
- Verify exit codes, output messages, and generated files

## MOBI/AZW3 Stubs

Initial implementation provides format detection and a "not yet supported" message:

```rust
pub struct MobiFixer;
impl FileFixer for MobiFixer {
    fn detect(data: &[u8]) -> bool {
        // Check for MOBI magic bytes
        data.len() > 60 && &data[60..68] == b"BOOKMOBI"
    }
    fn fix(&self, _data: &[u8], _options: &FixOptions) -> Result<FixOutput> {
        Err(KindleFixError::UnsupportedFormat("MOBI support coming soon".into()))
    }
}
```

Future MOBI/AZW3 fixes would likely focus on metadata corrections rather than the encoding issues specific to EPUB.

## Prior Art

- Original web app: https://kindle-epub-fix.netlify.app
- Source: `kindle-epub-fix/` (vanilla JS, UNLICENSE)
- Version history: v1.0 (Jul 2022) through v1.3 (Jan 2023)
