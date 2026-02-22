# Kindle File Fix Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a cross-platform CLI tool and Tauri GUI app in Rust that fixes EPUB files for Kindle compatibility, with extensible architecture for future MOBI/AZW3 support.

**Architecture:** Pure Rust monorepo with 3 Cargo workspace crates: `kindle-fix-core` (library with all fix logic), `kindle-fix-cli` (clap-based CLI binary), and `kindle-fix-gui` (Tauri desktop app with vanilla TS frontend). The core uses a `FileFixer` trait for format extensibility.

**Tech Stack:** Rust 2021 edition, `zip` crate for EPUB I/O, `quick-xml` for XML parsing, `regex` for encoding detection, `clap` for CLI, Tauri v2 for GUI, vanilla TypeScript for frontend.

**Reference:** Original JavaScript source at `../kindle-epub-fix/script.js` (340 lines). Design doc at `docs/plans/2026-02-22-kindle-file-fix-design.md`.

---

## Task 1: Project Scaffolding

**Files:**
- Create: `Cargo.toml` (workspace root)
- Create: `crates/kindle-fix-core/Cargo.toml`
- Create: `crates/kindle-fix-core/src/lib.rs`
- Create: `crates/kindle-fix-cli/Cargo.toml`
- Create: `crates/kindle-fix-cli/src/main.rs`
- Create: `.gitignore`
- Create: `LICENSE`
- Create: `rustfmt.toml`

**Step 1: Create workspace Cargo.toml**

```toml
# Cargo.toml (workspace root)
[workspace]
members = [
    "crates/kindle-fix-core",
    "crates/kindle-fix-cli",
]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"
authors = ["Klaus <klausdk1999@users.noreply.github.com>"]
license = "Apache-2.0"
repository = "https://github.com/Klausdk1999/kindle-file-fix"
```

**Step 2: Create core crate Cargo.toml**

```toml
# crates/kindle-fix-core/Cargo.toml
[package]
name = "kindle-fix-core"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true
description = "Core library for fixing Kindle-incompatible ebook files"

[dependencies]
zip = "2"
quick-xml = "0.37"
regex = "1"
thiserror = "2"
log = "0.4"

[dev-dependencies]
tempfile = "3"
```

**Step 3: Create CLI crate Cargo.toml**

```toml
# crates/kindle-fix-cli/Cargo.toml
[package]
name = "kindle-file-fix"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true
description = "CLI tool to fix Kindle-incompatible ebook files"

[[bin]]
name = "kindle-file-fix"
path = "src/main.rs"

[dependencies]
kindle-fix-core = { path = "../kindle-fix-core" }
clap = { version = "4", features = ["derive"] }
colored = "3"
indicatif = "0.17"
dialoguer = "0.11"
env_logger = "0.11"
log = "0.4"
```

**Step 4: Create minimal lib.rs and main.rs**

```rust
// crates/kindle-fix-core/src/lib.rs
//! Core library for fixing Kindle-incompatible ebook files.

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_set() {
        assert!(!version().is_empty());
    }
}
```

```rust
// crates/kindle-fix-cli/src/main.rs
fn main() {
    println!("kindle-file-fix v{}", kindle_fix_core::version());
}
```

**Step 5: Create .gitignore**

```
/target
*.swp
*.swo
.idea/
.vscode/
*.log
```

**Step 6: Create LICENSE (Apache 2.0)**

Use the standard Apache 2.0 license text with copyright line:
`Copyright 2026 Klaus`

**Step 7: Create rustfmt.toml**

```toml
max_width = 100
use_field_init_shorthand = true
```

**Step 8: Verify the workspace builds**

Run: `cargo build`
Expected: Compiles successfully with no errors.

Run: `cargo test`
Expected: 1 test passes (`version_is_set`).

**Step 9: Commit**

```bash
git add -A
git commit -m "feat: scaffold Cargo workspace with core and CLI crates"
```

---

## Task 2: Core Types and Error Handling

**Files:**
- Create: `crates/kindle-fix-core/src/error.rs`
- Create: `crates/kindle-fix-core/src/types.rs`
- Modify: `crates/kindle-fix-core/src/lib.rs`

**Step 1: Write tests for types**

```rust
// Add to lib.rs tests or create inline tests in types.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fix_options_defaults() {
        let opts = FixOptions::default();
        assert!(opts.language.is_none());
        assert!(!opts.keep_name);
        assert!(!opts.dry_run);
    }

    #[test]
    fn fix_report_starts_empty() {
        let report = FixReport::new("test.epub".into(), FileFormat::Epub);
        assert_eq!(report.filename, "test.epub");
        assert!(report.fixes_applied.is_empty());
        assert!(report.warnings.is_empty());
    }

    #[test]
    fn file_format_display() {
        assert_eq!(format!("{}", FileFormat::Epub), "EPUB");
        assert_eq!(format!("{}", FileFormat::Mobi), "MOBI");
        assert_eq!(format!("{}", FileFormat::Azw3), "AZW3");
        assert_eq!(format!("{}", FileFormat::Unknown), "Unknown");
    }

    #[test]
    fn error_display() {
        let err = KindleFixError::InvalidEpub("missing mimetype".into());
        assert!(err.to_string().contains("missing mimetype"));
    }
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test`
Expected: FAIL — types don't exist yet.

**Step 3: Implement error.rs**

```rust
// crates/kindle-fix-core/src/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum KindleFixError {
    #[error("Invalid EPUB: {0}")]
    InvalidEpub(String),

    #[error("ZIP error: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("XML error: {0}")]
    Xml(#[from] quick_xml::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
}

pub type Result<T> = std::result::Result<T, KindleFixError>;
```

**Step 4: Implement types.rs**

```rust
// crates/kindle-fix-core/src/types.rs
use std::fmt;

#[derive(Debug, Clone, Default)]
pub struct FixOptions {
    /// Override language code (skips interactive prompt)
    pub language: Option<String>,
    /// Keep original filename (no prefix)
    pub keep_name: bool,
    /// Report only, don't produce output data
    pub dry_run: bool,
}

#[derive(Debug, Clone)]
pub struct FixReport {
    pub filename: String,
    pub format: FileFormat,
    pub fixes_applied: Vec<FixDescription>,
    pub warnings: Vec<String>,
}

impl FixReport {
    pub fn new(filename: String, format: FileFormat) -> Self {
        Self {
            filename,
            format,
            fixes_applied: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn has_fixes(&self) -> bool {
        !self.fixes_applied.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct FixDescription {
    pub name: String,
    pub details: String,
    pub files_affected: usize,
}

#[derive(Debug, Clone)]
pub struct FixOutput {
    pub data: Vec<u8>,
    pub report: FixReport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileFormat {
    Epub,
    Mobi,
    Azw3,
    Unknown,
}

impl fmt::Display for FileFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileFormat::Epub => write!(f, "EPUB"),
            FileFormat::Mobi => write!(f, "MOBI"),
            FileFormat::Azw3 => write!(f, "AZW3"),
            FileFormat::Unknown => write!(f, "Unknown"),
        }
    }
}
```

**Step 5: Update lib.rs to export modules**

```rust
// crates/kindle-fix-core/src/lib.rs
//! Core library for fixing Kindle-incompatible ebook files.

pub mod error;
pub mod types;

pub use error::{KindleFixError, Result};
pub use types::{FileFormat, FixDescription, FixOptions, FixOutput, FixReport};

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
```

**Step 6: Run tests to verify they pass**

Run: `cargo test`
Expected: All tests PASS.

**Step 7: Commit**

```bash
git add -A
git commit -m "feat(core): add types, error handling, and FixReport/FixOptions"
```

---

## Task 3: EPUB Reader

**Files:**
- Create: `crates/kindle-fix-core/src/formats/mod.rs`
- Create: `crates/kindle-fix-core/src/formats/epub/mod.rs`
- Create: `crates/kindle-fix-core/src/formats/epub/reader.rs`
- Create: `tests/fixtures/` (with a minimal test EPUB)
- Modify: `crates/kindle-fix-core/src/lib.rs`

**Step 1: Create a minimal test EPUB fixture programmatically**

Create a helper in tests that builds minimal EPUBs. We'll use this throughout all tasks.

```rust
// crates/kindle-fix-core/tests/helpers/mod.rs

use std::io::{Cursor, Write};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

/// Build a minimal valid EPUB as bytes.
/// `files` is a list of (filename, content) pairs.
/// Automatically adds mimetype if not present.
pub fn build_epub(files: &[(&str, &str)]) -> Vec<u8> {
    let buf = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(buf);

    // Write mimetype first, uncompressed (EPUB spec)
    let has_mimetype = files.iter().any(|(name, _)| *name == "mimetype");
    if !has_mimetype {
        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        zip.start_file("mimetype", options).unwrap();
        zip.write_all(b"application/epub+zip").unwrap();
    }

    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    for (name, content) in files {
        if *name == "mimetype" {
            let stored = SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Stored);
            zip.start_file(*name, stored).unwrap();
        } else {
            zip.start_file(*name, options).unwrap();
        }
        zip.write_all(content.as_bytes()).unwrap();
    }

    zip.finish().unwrap().into_inner()
}

/// Minimal container.xml pointing to content.opf
pub const CONTAINER_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>"#;

/// Minimal OPF with language
pub fn opf_with_language(lang: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:title>Test Book</dc:title>
    <dc:language>{lang}</dc:language>
  </metadata>
  <manifest/>
  <spine/>
</package>"#
    )
}

/// Minimal OPF without language
pub fn opf_without_language() -> String {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:title>Test Book</dc:title>
  </metadata>
  <manifest/>
  <spine/>
</package>"#
        .to_string()
}
```

**Step 2: Write failing test for EPUB reader**

```rust
// crates/kindle-fix-core/tests/epub_reader_test.rs
mod helpers;

use kindle_fix_core::formats::epub::reader::EpubReader;

#[test]
fn reads_text_files_from_epub() {
    let epub_bytes = helpers::build_epub(&[
        ("META-INF/container.xml", helpers::CONTAINER_XML),
        ("OEBPS/content.opf", &helpers::opf_with_language("en")),
        ("OEBPS/chapter1.xhtml", "<html><body>Hello</body></html>"),
    ]);

    let reader = EpubReader::from_bytes(&epub_bytes).unwrap();
    assert!(reader.text_files().contains_key("META-INF/container.xml"));
    assert!(reader.text_files().contains_key("OEBPS/content.opf"));
    assert!(reader.text_files().contains_key("OEBPS/chapter1.xhtml"));
    assert!(reader.text_files().contains_key("mimetype"));
    assert_eq!(reader.text_files().len(), 4);
}

#[test]
fn separates_binary_files() {
    let buf = {
        use std::io::{Cursor, Write};
        use zip::write::SimpleFileOptions;
        use zip::ZipWriter;

        let cursor = Cursor::new(Vec::new());
        let mut zip = ZipWriter::new(cursor);
        let stored = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        let deflated = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        zip.start_file("mimetype", stored).unwrap();
        zip.write_all(b"application/epub+zip").unwrap();
        zip.start_file("OEBPS/chapter1.xhtml", deflated).unwrap();
        zip.write_all(b"<html/>").unwrap();
        zip.start_file("OEBPS/image.png", deflated).unwrap();
        zip.write_all(&[0x89, 0x50, 0x4E, 0x47]).unwrap(); // PNG header bytes
        zip.finish().unwrap().into_inner()
    };

    let reader = EpubReader::from_bytes(&buf).unwrap();
    assert!(reader.text_files().contains_key("OEBPS/chapter1.xhtml"));
    assert!(reader.binary_files().contains_key("OEBPS/image.png"));
    assert!(!reader.text_files().contains_key("OEBPS/image.png"));
}
```

**Step 3: Run tests to verify they fail**

Run: `cargo test`
Expected: FAIL — `formats` module doesn't exist.

**Step 4: Implement the EPUB reader**

```rust
// crates/kindle-fix-core/src/formats/mod.rs
pub mod epub;
```

```rust
// crates/kindle-fix-core/src/formats/epub/mod.rs
pub mod reader;
```

```rust
// crates/kindle-fix-core/src/formats/epub/reader.rs
use std::collections::HashMap;
use std::io::{Cursor, Read};

use crate::error::Result;
use crate::KindleFixError;

/// Text file extensions that should be read as UTF-8 strings.
const TEXT_EXTENSIONS: &[&str] = &[
    "html", "xhtml", "htm", "xml", "svg", "css", "opf", "ncx",
];

/// Special filenames (no extension) that are text.
const TEXT_FILENAMES: &[&str] = &["mimetype"];

pub struct EpubReader {
    text_files: HashMap<String, String>,
    binary_files: HashMap<String, Vec<u8>>,
}

impl EpubReader {
    /// Parse an EPUB from raw bytes.
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        let cursor = Cursor::new(data);
        let mut archive = zip::ZipArchive::new(cursor)?;

        let mut text_files = HashMap::new();
        let mut binary_files = HashMap::new();

        for i in 0..archive.len() {
            let mut entry = archive.by_index(i)?;
            let name = entry.name().to_string();

            // Skip directories
            if entry.is_dir() {
                continue;
            }

            if is_text_file(&name) {
                let mut content = String::new();
                entry.read_to_string(&mut content)?;
                text_files.insert(name, content);
            } else {
                let mut content = Vec::new();
                entry.read_to_end(&mut content)?;
                binary_files.insert(name, content);
            }
        }

        Ok(Self {
            text_files,
            binary_files,
        })
    }

    pub fn text_files(&self) -> &HashMap<String, String> {
        &self.text_files
    }

    pub fn text_files_mut(&mut self) -> &mut HashMap<String, String> {
        &mut self.text_files
    }

    pub fn binary_files(&self) -> &HashMap<String, Vec<u8>> {
        &self.binary_files
    }

    pub fn binary_files_mut(&mut self) -> &mut HashMap<String, Vec<u8>> {
        &mut self.binary_files
    }

    /// Consume the reader and return owned text and binary file maps.
    pub fn into_parts(self) -> (HashMap<String, String>, HashMap<String, Vec<u8>>) {
        (self.text_files, self.binary_files)
    }
}

fn is_text_file(filename: &str) -> bool {
    // Check special filenames
    let basename = filename.rsplit('/').next().unwrap_or(filename);
    if TEXT_FILENAMES.contains(&basename) {
        return true;
    }

    // Check extension
    if let Some(ext) = filename.rsplit('.').next() {
        TEXT_EXTENSIONS.contains(&ext.to_lowercase().as_str())
    } else {
        false
    }
}
```

**Step 5: Update lib.rs**

```rust
// crates/kindle-fix-core/src/lib.rs
//! Core library for fixing Kindle-incompatible ebook files.

pub mod error;
pub mod formats;
pub mod types;

pub use error::{KindleFixError, Result};
pub use types::{FileFormat, FixDescription, FixOptions, FixOutput, FixReport};

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
```

**Step 6: Run tests to verify they pass**

Run: `cargo test`
Expected: All tests PASS.

**Step 7: Commit**

```bash
git add -A
git commit -m "feat(core): add EPUB reader with text/binary file separation"
```

---

## Task 4: EPUB Writer

**Files:**
- Create: `crates/kindle-fix-core/src/formats/epub/writer.rs`
- Modify: `crates/kindle-fix-core/src/formats/epub/mod.rs`

**Step 1: Write failing test for EPUB writer**

```rust
// crates/kindle-fix-core/tests/epub_writer_test.rs
mod helpers;

use kindle_fix_core::formats::epub::reader::EpubReader;
use kindle_fix_core::formats::epub::writer::EpubWriter;
use std::collections::HashMap;

#[test]
fn roundtrip_preserves_content() {
    let original = helpers::build_epub(&[
        ("META-INF/container.xml", helpers::CONTAINER_XML),
        ("OEBPS/content.opf", &helpers::opf_with_language("en")),
        ("OEBPS/chapter1.xhtml", "<html><body>Hello</body></html>"),
    ]);

    // Read
    let reader = EpubReader::from_bytes(&original).unwrap();
    let (text_files, binary_files) = reader.into_parts();

    // Write
    let output = EpubWriter::write(&text_files, &binary_files).unwrap();

    // Read again
    let reader2 = EpubReader::from_bytes(&output).unwrap();
    assert_eq!(
        reader2.text_files().get("OEBPS/chapter1.xhtml").unwrap(),
        "<html><body>Hello</body></html>"
    );
    assert_eq!(
        reader2.text_files().get("mimetype").unwrap(),
        "application/epub+zip"
    );
}

#[test]
fn mimetype_is_first_and_uncompressed() {
    let mut text_files = HashMap::new();
    text_files.insert("mimetype".to_string(), "application/epub+zip".to_string());
    text_files.insert("OEBPS/chapter.xhtml".to_string(), "<html/>".to_string());

    let output = EpubWriter::write(&text_files, &HashMap::new()).unwrap();

    // Verify mimetype is stored (not compressed) by checking the ZIP
    let cursor = std::io::Cursor::new(&output);
    let archive = zip::ZipArchive::new(cursor).unwrap();
    let mimetype_entry = archive.by_name("mimetype").unwrap();
    assert_eq!(
        mimetype_entry.compression(),
        zip::CompressionMethod::Stored
    );
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test`
Expected: FAIL — writer module doesn't exist.

**Step 3: Implement the EPUB writer**

```rust
// crates/kindle-fix-core/src/formats/epub/writer.rs
use std::collections::HashMap;
use std::io::{Cursor, Write};

use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use crate::error::Result;

pub struct EpubWriter;

impl EpubWriter {
    /// Write an EPUB from text and binary file maps.
    /// Mimetype is written first and uncompressed per EPUB spec.
    pub fn write(
        text_files: &HashMap<String, String>,
        binary_files: &HashMap<String, Vec<u8>>,
    ) -> Result<Vec<u8>> {
        let buf = Cursor::new(Vec::new());
        let mut zip = ZipWriter::new(buf);

        let stored = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        let deflated = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        // Write mimetype FIRST, uncompressed (EPUB spec requirement)
        if let Some(mimetype) = text_files.get("mimetype") {
            zip.start_file("mimetype", stored)?;
            zip.write_all(mimetype.as_bytes())?;
        }

        // Write remaining text files
        for (name, content) in text_files {
            if name == "mimetype" {
                continue;
            }
            zip.start_file(name.as_str(), deflated)?;
            zip.write_all(content.as_bytes())?;
        }

        // Write binary files
        for (name, content) in binary_files {
            zip.start_file(name.as_str(), deflated)?;
            zip.write_all(content)?;
        }

        let cursor = zip.finish()?;
        Ok(cursor.into_inner())
    }
}
```

**Step 4: Update epub/mod.rs**

```rust
// crates/kindle-fix-core/src/formats/epub/mod.rs
pub mod reader;
pub mod writer;
```

**Step 5: Run tests to verify they pass**

Run: `cargo test`
Expected: All tests PASS.

**Step 6: Commit**

```bash
git add -A
git commit -m "feat(core): add EPUB writer with mimetype-first ordering"
```

---

## Task 5: Encoding Fix

**Files:**
- Create: `crates/kindle-fix-core/src/formats/epub/fixes/mod.rs`
- Create: `crates/kindle-fix-core/src/formats/epub/fixes/encoding.rs`
- Modify: `crates/kindle-fix-core/src/formats/epub/mod.rs`

**Step 1: Write failing tests**

```rust
// crates/kindle-fix-core/tests/fix_encoding_test.rs
mod helpers;

use std::collections::HashMap;
use kindle_fix_core::formats::epub::fixes::encoding::fix_encoding;

#[test]
fn adds_encoding_to_xhtml_without_declaration() {
    let mut files = HashMap::new();
    files.insert(
        "chapter1.xhtml".to_string(),
        "<html><body>Hello</body></html>".to_string(),
    );

    let fixes = fix_encoding(&mut files);
    assert_eq!(fixes.len(), 1);
    assert!(files["chapter1.xhtml"].starts_with("<?xml version=\"1.0\" encoding=\"utf-8\"?>"));
}

#[test]
fn skips_files_with_existing_encoding() {
    let mut files = HashMap::new();
    files.insert(
        "chapter1.xhtml".to_string(),
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<html><body>Hello</body></html>".to_string(),
    );

    let fixes = fix_encoding(&mut files);
    assert!(fixes.is_empty());
}

#[test]
fn handles_single_quote_encoding() {
    let mut files = HashMap::new();
    files.insert(
        "chapter1.xhtml".to_string(),
        "<?xml version='1.0' encoding='utf-8'?>\n<html/>".to_string(),
    );

    let fixes = fix_encoding(&mut files);
    assert!(fixes.is_empty());
}

#[test]
fn only_processes_html_and_xhtml() {
    let mut files = HashMap::new();
    files.insert("style.css".to_string(), "body { color: red }".to_string());
    files.insert("content.opf".to_string(), "<package/>".to_string());
    files.insert(
        "chapter.xhtml".to_string(),
        "<html/>".to_string(),
    );

    let fixes = fix_encoding(&mut files);
    assert_eq!(fixes.len(), 1);
    // CSS and OPF should be untouched
    assert_eq!(files["style.css"], "body { color: red }");
    assert_eq!(files["content.opf"], "<package/>");
}

#[test]
fn handles_leading_whitespace() {
    let mut files = HashMap::new();
    files.insert(
        "chapter1.xhtml".to_string(),
        "  \n  <html><body>Hello</body></html>".to_string(),
    );

    let fixes = fix_encoding(&mut files);
    assert_eq!(fixes.len(), 1);
    assert!(files["chapter1.xhtml"].starts_with("<?xml version=\"1.0\" encoding=\"utf-8\"?>"));
}

#[test]
fn handles_htm_extension() {
    let mut files = HashMap::new();
    files.insert(
        "chapter1.htm".to_string(),
        "<html/>".to_string(),
    );

    let fixes = fix_encoding(&mut files);
    assert_eq!(fixes.len(), 1);
}

#[test]
fn handles_html_extension() {
    let mut files = HashMap::new();
    files.insert(
        "chapter1.html".to_string(),
        "<html/>".to_string(),
    );

    let fixes = fix_encoding(&mut files);
    assert_eq!(fixes.len(), 1);
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test fix_encoding`
Expected: FAIL — module doesn't exist.

**Step 3: Implement encoding fix**

```rust
// crates/kindle-fix-core/src/formats/epub/fixes/mod.rs
pub mod encoding;

/// Check if a filename has an HTML/XHTML extension.
pub(crate) fn is_html_file(filename: &str) -> bool {
    let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
    matches!(ext.as_str(), "html" | "xhtml" | "htm")
}
```

```rust
// crates/kindle-fix-core/src/formats/epub/fixes/encoding.rs
use std::collections::HashMap;

use regex::Regex;

use super::is_html_file;

const ENCODING_DECLARATION: &str = r#"<?xml version="1.0" encoding="utf-8"?>"#;

/// Fix missing UTF-8 encoding declarations in HTML/XHTML files.
/// Returns a list of filenames that were fixed.
pub fn fix_encoding(files: &mut HashMap<String, String>) -> Vec<String> {
    let regex = Regex::new(
        r#"^<\?xml\s+version=["'][\d.]+["']\s+encoding=["'][a-zA-Z\d\-.]+["'].*?\?>"#,
    )
    .expect("encoding regex is valid");

    let mut fixed = Vec::new();

    let filenames: Vec<String> = files.keys().cloned().collect();
    for filename in filenames {
        if !is_html_file(&filename) {
            continue;
        }

        let content = &files[&filename];
        let trimmed = content.trim_start();

        if !regex.is_match(trimmed) {
            let new_content = format!("{}\n{}", ENCODING_DECLARATION, trimmed);
            files.insert(filename.clone(), new_content);
            fixed.push(filename);
        }
    }

    fixed
}
```

**Step 4: Update epub/mod.rs**

```rust
// crates/kindle-fix-core/src/formats/epub/mod.rs
pub mod fixes;
pub mod reader;
pub mod writer;
```

**Step 5: Run tests to verify they pass**

Run: `cargo test fix_encoding`
Expected: All tests PASS.

**Step 6: Commit**

```bash
git add -A
git commit -m "feat(core): add UTF-8 encoding declaration fix"
```

---

## Task 6: Body ID Link Fix

**Files:**
- Create: `crates/kindle-fix-core/src/formats/epub/fixes/body_id.rs`
- Modify: `crates/kindle-fix-core/src/formats/epub/fixes/mod.rs`

**Step 1: Write failing tests**

```rust
// crates/kindle-fix-core/tests/fix_body_id_test.rs
mod helpers;

use std::collections::HashMap;
use kindle_fix_core::formats::epub::fixes::body_id::fix_body_id_links;

#[test]
fn replaces_body_id_references() {
    let mut files = HashMap::new();
    files.insert(
        "OEBPS/chapter1.xhtml".to_string(),
        r#"<html><body id="chapter1body">Content</body></html>"#.to_string(),
    );
    files.insert(
        "OEBPS/toc.ncx".to_string(),
        r#"<navPoint><content src="chapter1.xhtml#chapter1body"/></navPoint>"#.to_string(),
    );

    let fixes = fix_body_id_links(&mut files);
    assert_eq!(fixes.len(), 1);
    assert!(files["OEBPS/toc.ncx"].contains("chapter1.xhtml\""));
    assert!(!files["OEBPS/toc.ncx"].contains("#chapter1body"));
}

#[test]
fn ignores_non_body_id_hashes() {
    let mut files = HashMap::new();
    files.insert(
        "OEBPS/chapter1.xhtml".to_string(),
        r#"<html><body>Content <div id="section1">Section</div></body></html>"#.to_string(),
    );
    files.insert(
        "OEBPS/toc.ncx".to_string(),
        r#"<navPoint><content src="chapter1.xhtml#section1"/></navPoint>"#.to_string(),
    );

    let fixes = fix_body_id_links(&mut files);
    assert!(fixes.is_empty());
    assert!(files["OEBPS/toc.ncx"].contains("#section1"));
}

#[test]
fn skips_body_without_id() {
    let mut files = HashMap::new();
    files.insert(
        "OEBPS/chapter1.xhtml".to_string(),
        r#"<html><body>Content</body></html>"#.to_string(),
    );

    let fixes = fix_body_id_links(&mut files);
    assert!(fixes.is_empty());
}

#[test]
fn handles_multiple_files_with_body_ids() {
    let mut files = HashMap::new();
    files.insert(
        "OEBPS/ch1.xhtml".to_string(),
        r#"<html><body id="b1">Ch1</body></html>"#.to_string(),
    );
    files.insert(
        "OEBPS/ch2.xhtml".to_string(),
        r#"<html><body id="b2">Ch2</body></html>"#.to_string(),
    );
    files.insert(
        "OEBPS/toc.ncx".to_string(),
        r#"<a href="ch1.xhtml#b1"/><a href="ch2.xhtml#b2"/>"#.to_string(),
    );

    let fixes = fix_body_id_links(&mut files);
    assert_eq!(fixes.len(), 2);
    assert!(files["OEBPS/toc.ncx"].contains("ch1.xhtml\""));
    assert!(files["OEBPS/toc.ncx"].contains("ch2.xhtml\""));
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test fix_body_id`
Expected: FAIL — module doesn't exist.

**Step 3: Implement body ID link fix**

```rust
// crates/kindle-fix-core/src/formats/epub/fixes/body_id.rs
use std::collections::HashMap;

use regex::Regex;

use super::is_html_file;

/// Fix body ID link references that Kindle rejects as unresolved hyperlinks.
///
/// Scans HTML/XHTML files for `<body id="...">` elements, then replaces all
/// references like `filename.html#bodyid` with just `filename.html`.
///
/// Returns a list of descriptions of replacements made.
pub fn fix_body_id_links(files: &mut HashMap<String, String>) -> Vec<String> {
    // Regex to find <body ... id="value" ...> in HTML content
    let body_id_regex =
        Regex::new(r#"<body\b[^>]*\bid\s*=\s*["']([^"']+)["'][^>]*>"#).expect("valid regex");

    // First pass: collect body IDs and their file basenames
    let mut body_id_map: Vec<(String, String)> = Vec::new(); // (src_with_hash, target_without_hash)

    let filenames: Vec<String> = files.keys().cloned().collect();
    for filename in &filenames {
        if !is_html_file(filename) {
            continue;
        }

        let content = &files[filename];
        if let Some(caps) = body_id_regex.captures(content) {
            let body_id = &caps[1];
            let basename = filename.rsplit('/').next().unwrap_or(filename);
            let src = format!("{}#{}", basename, body_id);
            body_id_map.push((src, basename.to_string()));
        }
    }

    if body_id_map.is_empty() {
        return Vec::new();
    }

    // Second pass: replace all references across all files
    let mut fixes = Vec::new();

    for filename in &filenames {
        let content = files.get(filename).unwrap().clone();
        let mut modified = content.clone();

        for (src, target) in &body_id_map {
            if modified.contains(src.as_str()) {
                modified = modified.replace(src.as_str(), target.as_str());
                fixes.push(format!(
                    "Replaced link target {} with {} in {}",
                    src, target, filename
                ));
            }
        }

        if modified != content {
            files.insert(filename.clone(), modified);
        }
    }

    fixes
}
```

**Step 4: Update fixes/mod.rs**

```rust
// crates/kindle-fix-core/src/formats/epub/fixes/mod.rs
pub mod body_id;
pub mod encoding;

/// Check if a filename has an HTML/XHTML extension.
pub(crate) fn is_html_file(filename: &str) -> bool {
    let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
    matches!(ext.as_str(), "html" | "xhtml" | "htm")
}
```

**Step 5: Run tests to verify they pass**

Run: `cargo test fix_body_id`
Expected: All tests PASS.

**Step 6: Commit**

```bash
git add -A
git commit -m "feat(core): add body ID link reference fix"
```

---

## Task 7: Language Tag Fix

**Files:**
- Create: `crates/kindle-fix-core/src/formats/epub/fixes/language.rs`
- Modify: `crates/kindle-fix-core/src/formats/epub/fixes/mod.rs`

**Step 1: Write failing tests**

```rust
// crates/kindle-fix-core/tests/fix_language_test.rs
mod helpers;

use std::collections::HashMap;
use kindle_fix_core::formats::epub::fixes::language::{
    fix_language, LanguageFixResult, SUPPORTED_LANGUAGES,
};

#[test]
fn detects_missing_language() {
    let mut files = HashMap::new();
    files.insert(
        "META-INF/container.xml".to_string(),
        helpers::CONTAINER_XML.to_string(),
    );
    files.insert(
        "OEBPS/content.opf".to_string(),
        helpers::opf_without_language(),
    );

    let result = fix_language(&mut files, Some("en".to_string()));
    match result {
        LanguageFixResult::Added(lang) => assert_eq!(lang, "en"),
        other => panic!("Expected Added, got {:?}", other),
    }
    assert!(files["OEBPS/content.opf"].contains("<dc:language>en</dc:language>"));
}

#[test]
fn detects_valid_language() {
    let mut files = HashMap::new();
    files.insert(
        "META-INF/container.xml".to_string(),
        helpers::CONTAINER_XML.to_string(),
    );
    files.insert(
        "OEBPS/content.opf".to_string(),
        helpers::opf_with_language("en"),
    );

    let result = fix_language(&mut files, None);
    match result {
        LanguageFixResult::Valid(lang) => assert_eq!(lang, "en"),
        other => panic!("Expected Valid, got {:?}", other),
    }
}

#[test]
fn detects_unsupported_language_with_override() {
    let mut files = HashMap::new();
    files.insert(
        "META-INF/container.xml".to_string(),
        helpers::CONTAINER_XML.to_string(),
    );
    files.insert(
        "OEBPS/content.opf".to_string(),
        helpers::opf_with_language("xx"),
    );

    let result = fix_language(&mut files, Some("en".to_string()));
    match result {
        LanguageFixResult::Changed { from, to } => {
            assert_eq!(from, "xx");
            assert_eq!(to, "en");
        }
        other => panic!("Expected Changed, got {:?}", other),
    }
}

#[test]
fn detects_unsupported_language_without_override() {
    let mut files = HashMap::new();
    files.insert(
        "META-INF/container.xml".to_string(),
        helpers::CONTAINER_XML.to_string(),
    );
    files.insert(
        "OEBPS/content.opf".to_string(),
        helpers::opf_with_language("xx"),
    );

    let result = fix_language(&mut files, None);
    match result {
        LanguageFixResult::Unsupported(lang) => assert_eq!(lang, "xx"),
        other => panic!("Expected Unsupported, got {:?}", other),
    }
}

#[test]
fn handles_regional_language_codes() {
    let mut files = HashMap::new();
    files.insert(
        "META-INF/container.xml".to_string(),
        helpers::CONTAINER_XML.to_string(),
    );
    files.insert(
        "OEBPS/content.opf".to_string(),
        helpers::opf_with_language("en-US"),
    );

    let result = fix_language(&mut files, None);
    match result {
        LanguageFixResult::Valid(lang) => assert_eq!(lang, "en-US"),
        other => panic!("Expected Valid, got {:?}", other),
    }
}

#[test]
fn case_insensitive_language_check() {
    let mut files = HashMap::new();
    files.insert(
        "META-INF/container.xml".to_string(),
        helpers::CONTAINER_XML.to_string(),
    );
    files.insert(
        "OEBPS/content.opf".to_string(),
        helpers::opf_with_language("EN"),
    );

    let result = fix_language(&mut files, None);
    match result {
        LanguageFixResult::Valid(lang) => assert_eq!(lang, "EN"),
        other => panic!("Expected Valid, got {:?}", other),
    }
}

#[test]
fn returns_error_on_missing_container_xml() {
    let mut files = HashMap::new();
    files.insert(
        "OEBPS/content.opf".to_string(),
        helpers::opf_with_language("en"),
    );

    let result = fix_language(&mut files, None);
    assert!(matches!(result, LanguageFixResult::Error(_)));
}

#[test]
fn supported_languages_includes_common_codes() {
    assert!(SUPPORTED_LANGUAGES.contains(&"en"));
    assert!(SUPPORTED_LANGUAGES.contains(&"fr"));
    assert!(SUPPORTED_LANGUAGES.contains(&"de"));
    assert!(SUPPORTED_LANGUAGES.contains(&"ja"));
    assert!(SUPPORTED_LANGUAGES.contains(&"eng"));
    assert!(SUPPORTED_LANGUAGES.contains(&"fra"));
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test fix_language`
Expected: FAIL — module doesn't exist.

**Step 3: Implement language fix**

```rust
// crates/kindle-fix-core/src/formats/epub/fixes/language.rs
use std::collections::HashMap;

use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::{Reader, Writer};

/// Supported Kindle language codes from https://kdp.amazon.com/en_US/help/topic/G200673300
pub const SUPPORTED_LANGUAGES: &[&str] = &[
    // ISO 639-1
    "af", "gsw", "ar", "eu", "nb", "br", "ca", "zh", "kw", "co", "da", "nl", "stq", "en", "fi",
    "fr", "fy", "gl", "de", "gu", "hi", "is", "ga", "it", "ja", "lb", "mr", "ml", "gv", "frr",
    "nn", "pl", "pt", "oc", "rm", "sco", "gd", "es", "sv", "ta", "cy",
    // ISO 639-2
    "afr", "ara", "eus", "baq", "nob", "bre", "cat", "zho", "chi", "cor", "cos", "dan", "nld",
    "dut", "eng", "fin", "fra", "fre", "fry", "glg", "deu", "ger", "guj", "hin", "isl", "ice",
    "gle", "ita", "jpn", "ltz", "mar", "mal", "glv", "nor", "nno", "por", "oci", "roh", "gla",
    "spa", "swe", "tam", "cym", "wel",
];

#[derive(Debug)]
pub enum LanguageFixResult {
    /// Language tag was present and valid.
    Valid(String),
    /// Language tag was missing, added with the given value.
    Added(String),
    /// Language tag was changed from one value to another.
    Changed { from: String, to: String },
    /// Language tag is unsupported and no override was provided.
    Unsupported(String),
    /// Could not process (missing files, parse errors, etc.)
    Error(String),
}

/// Simplify a language code by taking only the base language.
/// e.g. "en-US" -> "en", "fr-CA" -> "fr"
fn simplify_language(lang: &str) -> String {
    lang.split('-').next().unwrap_or(lang).to_lowercase()
}

/// Check if a language code is supported by Kindle.
fn is_supported(lang: &str) -> bool {
    let simplified = simplify_language(lang);
    SUPPORTED_LANGUAGES.contains(&simplified.as_str())
}

/// Find the OPF file path from container.xml.
fn find_opf_path(container_xml: &str) -> Option<String> {
    let mut reader = Reader::from_str(container_xml);
    loop {
        match reader.read_event() {
            Ok(Event::Empty(ref e)) | Ok(Event::Start(ref e)) if e.name().as_ref() == b"rootfile" => {
                let mut media_type = None;
                let mut full_path = None;
                for attr in e.attributes().flatten() {
                    match attr.key.as_ref() {
                        b"media-type" => {
                            media_type = Some(
                                String::from_utf8_lossy(&attr.value).to_string(),
                            );
                        }
                        b"full-path" => {
                            full_path = Some(
                                String::from_utf8_lossy(&attr.value).to_string(),
                            );
                        }
                        _ => {}
                    }
                }
                if media_type.as_deref() == Some("application/oebps-package+xml") {
                    return full_path;
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
    }
    None
}

/// Fix the language tag in an EPUB's OPF metadata.
///
/// - If the language tag is missing: adds it using `language_override` or returns an error.
/// - If the language tag is unsupported: replaces it with `language_override` or returns warning.
/// - If the language tag is valid: returns Valid.
pub fn fix_language(
    files: &mut HashMap<String, String>,
    language_override: Option<String>,
) -> LanguageFixResult {
    // Find container.xml
    let container = match files.get("META-INF/container.xml") {
        Some(c) => c.clone(),
        None => return LanguageFixResult::Error("Missing META-INF/container.xml".into()),
    };

    // Find OPF path
    let opf_path = match find_opf_path(&container) {
        Some(p) => p,
        None => return LanguageFixResult::Error("Could not find OPF file path in container.xml".into()),
    };

    // Get OPF content
    let opf_content = match files.get(&opf_path) {
        Some(c) => c.clone(),
        None => return LanguageFixResult::Error(format!("OPF file not found: {}", opf_path)),
    };

    // Parse OPF to find dc:language
    let current_language = extract_language(&opf_content);

    match current_language {
        None => {
            // Language tag missing
            let lang = language_override.unwrap_or_else(|| "en".to_string());
            match add_language_to_opf(&opf_content, &lang) {
                Some(new_opf) => {
                    files.insert(opf_path, new_opf);
                    LanguageFixResult::Added(lang)
                }
                None => LanguageFixResult::Error("Failed to add language tag to OPF".into()),
            }
        }
        Some(lang) => {
            if is_supported(&lang) {
                LanguageFixResult::Valid(lang)
            } else if let Some(override_lang) = language_override {
                // Replace unsupported language
                let new_opf = replace_language_in_opf(&opf_content, &override_lang);
                files.insert(opf_path, new_opf);
                LanguageFixResult::Changed {
                    from: lang,
                    to: override_lang,
                }
            } else {
                LanguageFixResult::Unsupported(lang)
            }
        }
    }
}

/// Extract the dc:language value from OPF XML content.
fn extract_language(opf: &str) -> Option<String> {
    let mut reader = Reader::from_str(opf);
    let mut in_language = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                let name = e.name();
                let local = name.as_ref();
                // Match dc:language or just language with dc namespace
                if local == b"dc:language" || local.ends_with(b":language") || local == b"language" {
                    in_language = true;
                }
            }
            Ok(Event::Text(ref e)) if in_language => {
                return Some(e.unescape().unwrap_or_default().trim().to_string());
            }
            Ok(Event::End(_)) if in_language => {
                in_language = false;
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
    }
    None
}

/// Replace the dc:language value in OPF content.
fn replace_language_in_opf(opf: &str, new_lang: &str) -> String {
    let mut reader = Reader::from_str(opf);
    let mut writer = Writer::new(Vec::new());
    let mut in_language = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                let name = e.name();
                let local = name.as_ref();
                if local == b"dc:language" || local.ends_with(b":language") || local == b"language" {
                    in_language = true;
                }
                writer.write_event(Event::Start(e.clone())).ok();
            }
            Ok(Event::Text(ref e)) if in_language => {
                writer
                    .write_event(Event::Text(BytesText::new(new_lang)))
                    .ok();
            }
            Ok(Event::End(ref e)) if in_language => {
                in_language = false;
                writer.write_event(Event::End(e.clone())).ok();
            }
            Ok(Event::Eof) => break,
            Ok(e) => {
                writer.write_event(e).ok();
            }
            Err(_) => break,
        }
    }

    String::from_utf8(writer.into_inner()).unwrap_or_else(|_| opf.to_string())
}

/// Add a dc:language element to OPF metadata.
fn add_language_to_opf(opf: &str, lang: &str) -> Option<String> {
    let mut reader = Reader::from_str(opf);
    let mut writer = Writer::new(Vec::new());
    let mut added = false;

    loop {
        match reader.read_event() {
            Ok(Event::End(ref e)) if !added && e.name().as_ref() == b"metadata" => {
                // Insert dc:language before closing metadata tag
                writer
                    .write_event(Event::Start(BytesStart::new("dc:language")))
                    .ok();
                writer
                    .write_event(Event::Text(BytesText::new(lang)))
                    .ok();
                writer
                    .write_event(Event::End(BytesEnd::new("dc:language")))
                    .ok();
                writer.write_event(Event::End(e.clone())).ok();
                added = true;
            }
            Ok(Event::Eof) => break,
            Ok(e) => {
                writer.write_event(e).ok();
            }
            Err(_) => break,
        }
    }

    if added {
        Some(String::from_utf8(writer.into_inner()).unwrap_or_else(|_| opf.to_string()))
    } else {
        None
    }
}
```

**Step 4: Update fixes/mod.rs**

```rust
pub mod body_id;
pub mod encoding;
pub mod language;

pub(crate) fn is_html_file(filename: &str) -> bool {
    let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
    matches!(ext.as_str(), "html" | "xhtml" | "htm")
}
```

**Step 5: Run tests to verify they pass**

Run: `cargo test fix_language`
Expected: All tests PASS.

**Step 6: Commit**

```bash
git add -A
git commit -m "feat(core): add language tag validation and fix"
```

---

## Task 8: Stray Image Fix

**Files:**
- Create: `crates/kindle-fix-core/src/formats/epub/fixes/stray_img.rs`
- Modify: `crates/kindle-fix-core/src/formats/epub/fixes/mod.rs`

**Step 1: Write failing tests**

```rust
// crates/kindle-fix-core/tests/fix_stray_img_test.rs
mod helpers;

use std::collections::HashMap;
use kindle_fix_core::formats::epub::fixes::stray_img::fix_stray_images;

#[test]
fn removes_img_without_src() {
    let mut files = HashMap::new();
    files.insert(
        "chapter.xhtml".to_string(),
        r#"<?xml version="1.0" encoding="utf-8"?><html><body><p>Text</p><img/><p>More</p></body></html>"#.to_string(),
    );

    let fixes = fix_stray_images(&mut files);
    assert_eq!(fixes.len(), 1);
    assert!(!files["chapter.xhtml"].contains("<img"));
}

#[test]
fn keeps_img_with_src() {
    let mut files = HashMap::new();
    files.insert(
        "chapter.xhtml".to_string(),
        r#"<?xml version="1.0" encoding="utf-8"?><html><body><img src="image.png"/></body></html>"#.to_string(),
    );

    let fixes = fix_stray_images(&mut files);
    assert!(fixes.is_empty());
    assert!(files["chapter.xhtml"].contains(r#"src="image.png""#));
}

#[test]
fn handles_multiple_stray_images() {
    let mut files = HashMap::new();
    files.insert(
        "chapter.xhtml".to_string(),
        r#"<?xml version="1.0" encoding="utf-8"?><html><body><img/><img src="ok.png"/><img/></body></html>"#.to_string(),
    );

    let fixes = fix_stray_images(&mut files);
    assert_eq!(fixes.len(), 1); // One fix description for the file
    // Should still have the img with src
    assert!(files["chapter.xhtml"].contains("ok.png"));
}

#[test]
fn only_processes_html_files() {
    let mut files = HashMap::new();
    files.insert("style.css".to_string(), "img { display: none }".to_string());
    files.insert(
        "chapter.xhtml".to_string(),
        r#"<?xml version="1.0" encoding="utf-8"?><html><body><img/></body></html>"#.to_string(),
    );

    let fixes = fix_stray_images(&mut files);
    assert_eq!(fixes.len(), 1);
    assert_eq!(files["style.css"], "img { display: none }");
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test fix_stray_img`
Expected: FAIL — module doesn't exist.

**Step 3: Implement stray image fix**

Note: Since Rust doesn't have a browser DOM, we use regex-based approach for removing `<img` tags without `src`. This is simpler and avoids pulling in a full HTML parser. The original JS uses DOMParser but the patterns are simple enough for regex.

```rust
// crates/kindle-fix-core/src/formats/epub/fixes/stray_img.rs
use std::collections::HashMap;

use regex::Regex;

use super::is_html_file;

/// Remove `<img>` tags that have no `src` attribute.
/// Returns a list of filenames where stray images were removed.
pub fn fix_stray_images(files: &mut HashMap<String, String>) -> Vec<String> {
    // Match self-closing <img ... /> or <img ...> that do NOT contain src=
    // We match img tags and check if they have src attribute
    let img_regex = Regex::new(r#"<img\b([^>]*)/?>"#).expect("valid regex");
    let src_regex = Regex::new(r#"\bsrc\s*="#).expect("valid regex");

    let mut fixed = Vec::new();

    let filenames: Vec<String> = files.keys().cloned().collect();
    for filename in filenames {
        if !is_html_file(&filename) {
            continue;
        }

        let content = files[&filename].clone();
        let mut removed_count = 0;

        let new_content = img_regex.replace_all(&content, |caps: &regex::Captures| {
            let attrs = &caps[1];
            if src_regex.is_match(attrs) {
                // Has src, keep it
                caps[0].to_string()
            } else {
                // No src, remove it
                removed_count += 1;
                String::new()
            }
        });

        if removed_count > 0 {
            files.insert(filename.clone(), new_content.to_string());
            fixed.push(filename);
        }
    }

    fixed
}
```

**Step 4: Update fixes/mod.rs**

```rust
pub mod body_id;
pub mod encoding;
pub mod language;
pub mod stray_img;

pub(crate) fn is_html_file(filename: &str) -> bool {
    let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
    matches!(ext.as_str(), "html" | "xhtml" | "htm")
}
```

**Step 5: Run tests to verify they pass**

Run: `cargo test fix_stray_img`
Expected: All tests PASS.

**Step 6: Commit**

```bash
git add -A
git commit -m "feat(core): add stray image tag removal fix"
```

---

## Task 9: EpubFixer — Orchestrate All Fixes

**Files:**
- Modify: `crates/kindle-fix-core/src/formats/epub/mod.rs`
- Modify: `crates/kindle-fix-core/src/formats/mod.rs`

**Step 1: Write failing integration test**

```rust
// crates/kindle-fix-core/tests/epub_fixer_test.rs
mod helpers;

use kindle_fix_core::formats::epub::EpubFixer;
use kindle_fix_core::formats::FileFixer;
use kindle_fix_core::types::FixOptions;

#[test]
fn fixes_epub_with_missing_encoding() {
    let epub = helpers::build_epub(&[
        ("META-INF/container.xml", helpers::CONTAINER_XML),
        ("OEBPS/content.opf", &helpers::opf_with_language("en")),
        ("OEBPS/chapter1.xhtml", "<html><body>Hello</body></html>"),
    ]);

    let fixer = EpubFixer;
    let output = fixer
        .fix(&epub, &FixOptions::default())
        .unwrap();

    assert!(!output.report.fixes_applied.is_empty());
    assert!(output.report.fixes_applied.iter().any(|f| f.name == "encoding"));
}

#[test]
fn detects_epub_format() {
    let epub = helpers::build_epub(&[
        ("META-INF/container.xml", helpers::CONTAINER_XML),
        ("OEBPS/content.opf", &helpers::opf_with_language("en")),
    ]);

    assert!(EpubFixer::detect(&epub));
}

#[test]
fn does_not_detect_random_zip() {
    use std::io::{Cursor, Write};
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    let cursor = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(cursor);
    let opts = SimpleFileOptions::default();
    zip.start_file("hello.txt", opts).unwrap();
    zip.write_all(b"hello").unwrap();
    let data = zip.finish().unwrap().into_inner();

    assert!(!EpubFixer::detect(&data));
}

#[test]
fn clean_epub_produces_no_fixes() {
    let epub = helpers::build_epub(&[
        ("META-INF/container.xml", helpers::CONTAINER_XML),
        ("OEBPS/content.opf", &helpers::opf_with_language("en")),
        (
            "OEBPS/chapter1.xhtml",
            "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<html><body>Hello</body></html>",
        ),
    ]);

    let fixer = EpubFixer;
    let output = fixer
        .fix(&epub, &FixOptions::default())
        .unwrap();

    assert!(output.report.fixes_applied.is_empty());
}

#[test]
fn output_is_valid_epub() {
    let epub = helpers::build_epub(&[
        ("META-INF/container.xml", helpers::CONTAINER_XML),
        ("OEBPS/content.opf", &helpers::opf_with_language("en")),
        ("OEBPS/chapter1.xhtml", "<html><body>Hello</body></html>"),
    ]);

    let fixer = EpubFixer;
    let output = fixer
        .fix(&epub, &FixOptions::default())
        .unwrap();

    // Should be readable as an EPUB again
    let reader = kindle_fix_core::formats::epub::reader::EpubReader::from_bytes(&output.data).unwrap();
    assert!(reader.text_files().contains_key("mimetype"));
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test epub_fixer`
Expected: FAIL — `EpubFixer` doesn't exist.

**Step 3: Implement EpubFixer and FileFixer trait**

```rust
// crates/kindle-fix-core/src/formats/mod.rs
pub mod epub;
pub mod mobi;
pub mod azw3;

use crate::error::Result;
use crate::types::{FixOptions, FixOutput};

/// Trait for format-specific file fixers.
pub trait FileFixer {
    /// Detect if the given data matches this format.
    fn detect(data: &[u8]) -> bool;

    /// Apply all fixes and return the fixed data with a report.
    fn fix(&self, data: &[u8], options: &FixOptions) -> Result<FixOutput>;
}
```

```rust
// crates/kindle-fix-core/src/formats/epub/mod.rs
pub mod fixes;
pub mod reader;
pub mod writer;

use std::io::Cursor;

use crate::error::Result;
use crate::formats::FileFixer;
use crate::types::{FileFormat, FixDescription, FixOptions, FixOutput, FixReport};

use self::fixes::body_id::fix_body_id_links;
use self::fixes::encoding::fix_encoding;
use self::fixes::language::{fix_language, LanguageFixResult};
use self::fixes::stray_img::fix_stray_images;
use self::reader::EpubReader;
use self::writer::EpubWriter;

pub struct EpubFixer;

impl FileFixer for EpubFixer {
    fn detect(data: &[u8]) -> bool {
        // EPUB is a ZIP with a "mimetype" entry containing "application/epub+zip"
        let cursor = Cursor::new(data);
        if let Ok(mut archive) = zip::ZipArchive::new(cursor) {
            if let Ok(mut entry) = archive.by_name("mimetype") {
                let mut content = String::new();
                use std::io::Read;
                if entry.read_to_string(&mut content).is_ok() {
                    return content.trim() == "application/epub+zip";
                }
            }
        }
        false
    }

    fn fix(&self, data: &[u8], options: &FixOptions) -> Result<FixOutput> {
        let reader = EpubReader::from_bytes(data)?;
        let (mut text_files, binary_files) = reader.into_parts();
        let mut report = FixReport::new(String::new(), FileFormat::Epub);

        // Fix 1: Body ID links
        let body_id_fixes = fix_body_id_links(&mut text_files);
        if !body_id_fixes.is_empty() {
            report.fixes_applied.push(FixDescription {
                name: "body_id".to_string(),
                details: format!("Removed {} body ID link reference(s)", body_id_fixes.len()),
                files_affected: body_id_fixes.len(),
            });
        }

        // Fix 2: Language
        let lang_result = fix_language(&mut text_files, options.language.clone());
        match &lang_result {
            LanguageFixResult::Added(lang) => {
                report.fixes_applied.push(FixDescription {
                    name: "language".to_string(),
                    details: format!("Added missing language tag: {}", lang),
                    files_affected: 1,
                });
            }
            LanguageFixResult::Changed { from, to } => {
                report.fixes_applied.push(FixDescription {
                    name: "language".to_string(),
                    details: format!("Changed language from {} to {}", from, to),
                    files_affected: 1,
                });
            }
            LanguageFixResult::Unsupported(lang) => {
                report.warnings.push(format!(
                    "Language '{}' is not supported by Kindle. Use --language to override.",
                    lang
                ));
            }
            LanguageFixResult::Error(msg) => {
                report.warnings.push(format!("Language check failed: {}", msg));
            }
            LanguageFixResult::Valid(_) => {}
        }

        // Fix 3: Stray images
        let stray_img_fixes = fix_stray_images(&mut text_files);
        if !stray_img_fixes.is_empty() {
            report.fixes_applied.push(FixDescription {
                name: "stray_img".to_string(),
                details: format!(
                    "Removed stray image tag(s) in {} file(s)",
                    stray_img_fixes.len()
                ),
                files_affected: stray_img_fixes.len(),
            });
        }

        // Fix 4: Encoding
        let encoding_fixes = fix_encoding(&mut text_files);
        if !encoding_fixes.is_empty() {
            report.fixes_applied.push(FixDescription {
                name: "encoding".to_string(),
                details: format!(
                    "Added UTF-8 encoding declaration to {} file(s)",
                    encoding_fixes.len()
                ),
                files_affected: encoding_fixes.len(),
            });
        }

        // Write output
        let output_data = if options.dry_run {
            Vec::new()
        } else {
            EpubWriter::write(&text_files, &binary_files)?
        };

        Ok(FixOutput {
            data: output_data,
            report,
        })
    }
}
```

**Step 4: Create MOBI and AZW3 stubs**

```rust
// crates/kindle-fix-core/src/formats/mobi/mod.rs
use crate::error::Result;
use crate::formats::FileFixer;
use crate::types::{FixOptions, FixOutput};
use crate::KindleFixError;

pub struct MobiFixer;

impl FileFixer for MobiFixer {
    fn detect(data: &[u8]) -> bool {
        // MOBI magic bytes at offset 60: "BOOKMOBI"
        data.len() > 68 && &data[60..68] == b"BOOKMOBI"
    }

    fn fix(&self, _data: &[u8], _options: &FixOptions) -> Result<FixOutput> {
        Err(KindleFixError::UnsupportedFormat(
            "MOBI support coming soon".into(),
        ))
    }
}
```

```rust
// crates/kindle-fix-core/src/formats/azw3/mod.rs
use crate::error::Result;
use crate::formats::FileFixer;
use crate::types::{FixOptions, FixOutput};
use crate::KindleFixError;

pub struct Azw3Fixer;

impl FileFixer for Azw3Fixer {
    fn detect(data: &[u8]) -> bool {
        // AZW3 uses the same PDB header as MOBI but with KF8 content
        // For now, detect via MOBI header — we'll refine this later
        data.len() > 68 && &data[60..68] == b"BOOKMOBI"
    }

    fn fix(&self, _data: &[u8], _options: &FixOptions) -> Result<FixOutput> {
        Err(KindleFixError::UnsupportedFormat(
            "AZW3 support coming soon".into(),
        ))
    }
}
```

**Step 5: Run tests to verify they pass**

Run: `cargo test`
Expected: All tests PASS.

**Step 6: Commit**

```bash
git add -A
git commit -m "feat(core): add EpubFixer orchestration and MOBI/AZW3 stubs"
```

---

## Task 10: Public API — `process_file()`

**Files:**
- Modify: `crates/kindle-fix-core/src/lib.rs`

**Step 1: Write failing test**

```rust
// crates/kindle-fix-core/tests/process_file_test.rs
mod helpers;

use kindle_fix_core::{process_file, FixOptions, FileFormat};

#[test]
fn process_epub_file() {
    let epub = helpers::build_epub(&[
        ("META-INF/container.xml", helpers::CONTAINER_XML),
        ("OEBPS/content.opf", &helpers::opf_with_language("en")),
        ("OEBPS/chapter1.xhtml", "<html><body>Hello</body></html>"),
    ]);

    let output = process_file(&epub, "test.epub", &FixOptions::default()).unwrap();
    assert_eq!(output.report.format, FileFormat::Epub);
    assert_eq!(output.report.filename, "test.epub");
    assert!(!output.data.is_empty());
}

#[test]
fn process_unknown_format() {
    let data = b"this is not an ebook";
    let result = process_file(data, "test.txt", &FixOptions::default());
    assert!(result.is_err());
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test process_file`
Expected: FAIL — `process_file` doesn't exist.

**Step 3: Implement process_file**

Add to `crates/kindle-fix-core/src/lib.rs`:

```rust
//! Core library for fixing Kindle-incompatible ebook files.

pub mod error;
pub mod formats;
pub mod types;

pub use error::{KindleFixError, Result};
pub use types::{FileFormat, FixDescription, FixOptions, FixOutput, FixReport};

use formats::epub::EpubFixer;
use formats::mobi::MobiFixer;
use formats::FileFixer;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Process a file, auto-detecting its format and applying all relevant fixes.
pub fn process_file(data: &[u8], filename: &str, options: &FixOptions) -> Result<FixOutput> {
    if EpubFixer::detect(data) {
        let fixer = EpubFixer;
        let mut output = fixer.fix(data, options)?;
        output.report.filename = filename.to_string();
        Ok(output)
    } else if MobiFixer::detect(data) {
        let fixer = MobiFixer;
        let mut output = fixer.fix(data, options)?;
        output.report.filename = filename.to_string();
        Ok(output)
    } else {
        Err(KindleFixError::UnsupportedFormat(format!(
            "Could not detect format of '{}'",
            filename
        )))
    }
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test`
Expected: All tests PASS.

**Step 5: Commit**

```bash
git add -A
git commit -m "feat(core): add process_file() public API with format auto-detection"
```

---

## Task 11: CLI Tool

**Files:**
- Modify: `crates/kindle-fix-cli/src/main.rs`
- Create: `crates/kindle-fix-cli/src/output.rs`

**Step 1: Implement CLI argument parsing and main logic**

```rust
// crates/kindle-fix-cli/src/main.rs
mod output;

use std::fs;
use std::path::{Path, PathBuf};

use clap::Parser;
use colored::Colorize;
use dialoguer::Input;

use kindle_fix_core::{process_file, FixOptions};

#[derive(Parser, Debug)]
#[command(
    name = "kindle-file-fix",
    about = "Fix ebook files for Kindle compatibility",
    version
)]
struct Cli {
    /// Input files or directories to process
    #[arg(required = true)]
    files: Vec<PathBuf>,

    /// Output directory (default: same as input file)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Set language code (skip interactive prompt)
    #[arg(short, long)]
    language: Option<String>,

    /// Keep original filename (no prefix)
    #[arg(long)]
    keep_name: bool,

    /// Show fixes without writing files
    #[arg(long)]
    dry_run: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Suppress output except errors
    #[arg(short, long)]
    quiet: bool,
}

fn main() {
    let cli = Cli::parse();

    if cli.verbose {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Debug)
            .init();
    } else {
        env_logger::init();
    }

    let files = collect_files(&cli.files);

    if files.is_empty() {
        eprintln!("{}", "No supported files found.".red());
        std::process::exit(1);
    }

    let mut total_fixes = 0;
    let mut processed = 0;
    let mut errors = 0;

    for path in &files {
        let filename = path.file_name().unwrap_or_default().to_string_lossy();

        if !cli.quiet {
            println!("{} {}", "Processing:".bold(), filename);
        }

        let data = match fs::read(path) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("  {} Could not read {}: {}", "[ERROR]".red(), filename, e);
                errors += 1;
                continue;
            }
        };

        // If no language override and the fixer needs one, prompt interactively
        let language = cli.language.clone().or_else(|| {
            // Try processing without language first to see if it's needed
            None
        });

        let options = FixOptions {
            language,
            keep_name: cli.keep_name,
            dry_run: cli.dry_run,
        };

        match process_file(&data, &filename, &options) {
            Ok(result) => {
                // Handle language warnings (may need interactive prompt)
                for warning in &result.report.warnings {
                    if warning.contains("not supported by Kindle") && cli.language.is_none() {
                        if !cli.quiet {
                            eprintln!("  {} {}", "[WARN]".yellow().bold(), warning);
                            // Prompt for language
                            if let Ok(lang) = Input::<String>::new()
                                .with_prompt("  Enter language code (e.g., en, fr, ja)")
                                .default("en".into())
                                .interact_text()
                            {
                                // Re-process with the language override
                                let new_options = FixOptions {
                                    language: Some(lang),
                                    keep_name: cli.keep_name,
                                    dry_run: cli.dry_run,
                                };
                                if let Ok(new_result) = process_file(&data, &filename, &new_options) {
                                    output::print_report(&new_result.report, cli.quiet);
                                    if !cli.dry_run {
                                        write_output(path, &new_result.data, cli.keep_name, &cli.output);
                                    }
                                    total_fixes += new_result.report.fixes_applied.len();
                                    processed += 1;
                                    continue;
                                }
                            }
                        }
                    }
                }

                output::print_report(&result.report, cli.quiet);

                if !cli.dry_run && !result.data.is_empty() {
                    write_output(path, &result.data, cli.keep_name, &cli.output);
                }

                total_fixes += result.report.fixes_applied.len();
                processed += 1;
            }
            Err(e) => {
                eprintln!("  {} {}", "[ERROR]".red().bold(), e);
                errors += 1;
            }
        }

        if !cli.quiet {
            println!();
        }
    }

    if !cli.quiet {
        println!(
            "{}",
            format!(
                "Processed {} file(s), {} fix(es) applied, {} error(s).",
                processed, total_fixes, errors
            )
            .bold()
        );
    }

    if errors > 0 {
        std::process::exit(1);
    }
}

fn collect_files(paths: &[PathBuf]) -> Vec<PathBuf> {
    let supported_extensions = ["epub", "mobi", "azw3"];
    let mut result = Vec::new();

    for path in paths {
        if path.is_dir() {
            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if let Some(ext) = p.extension() {
                        if supported_extensions.contains(&ext.to_string_lossy().to_lowercase().as_str()) {
                            result.push(p);
                        }
                    }
                }
            }
        } else if path.is_file() {
            result.push(path.clone());
        } else {
            eprintln!("{}: {} not found", "Warning".yellow(), path.display());
        }
    }

    result
}

fn write_output(input_path: &Path, data: &[u8], keep_name: bool, output_dir: &Option<PathBuf>) {
    let filename = input_path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy();

    let output_filename = if keep_name {
        filename.to_string()
    } else {
        format!("(fixed) {}", filename)
    };

    let output_path = if let Some(dir) = output_dir {
        fs::create_dir_all(dir).ok();
        dir.join(&output_filename)
    } else {
        input_path.with_file_name(&output_filename)
    };

    match fs::write(&output_path, data) {
        Ok(_) => {
            println!(
                "  {} {}",
                "Saved:".green().bold(),
                output_path.display()
            );
        }
        Err(e) => {
            eprintln!(
                "  {} Could not write {}: {}",
                "[ERROR]".red(),
                output_path.display(),
                e
            );
        }
    }
}
```

**Step 2: Implement output formatting**

```rust
// crates/kindle-fix-cli/src/output.rs
use colored::Colorize;
use kindle_fix_core::FixReport;

pub fn print_report(report: &FixReport, quiet: bool) {
    if quiet {
        return;
    }

    for fix in &report.fixes_applied {
        println!(
            "  {} {}",
            "[FIXED]".green().bold(),
            fix.details
        );
    }

    for warning in &report.warnings {
        println!("  {} {}", "[WARN]".yellow().bold(), warning);
    }

    if report.fixes_applied.is_empty() && report.warnings.is_empty() {
        println!("  {} No issues found", "[OK]".blue().bold());
    }
}
```

**Step 3: Verify it builds and runs**

Run: `cargo build`
Expected: Compiles successfully.

Run: `cargo run --bin kindle-file-fix -- --help`
Expected: Shows help text with all options.

**Step 4: Commit**

```bash
git add -A
git commit -m "feat(cli): add kindle-file-fix CLI with all options"
```

---

## Task 12: CLI Integration Test

**Files:**
- Create: `crates/kindle-fix-cli/tests/cli_test.rs`

**Step 1: Write CLI integration test**

```rust
// crates/kindle-fix-cli/tests/cli_test.rs
use std::io::Write;
use std::process::Command;

use tempfile::NamedTempFile;

fn build_test_epub() -> Vec<u8> {
    use std::io::Cursor;
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    let cursor = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(cursor);
    let stored = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Stored);
    let deflated = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    zip.start_file("mimetype", stored).unwrap();
    zip.write_all(b"application/epub+zip").unwrap();

    zip.start_file("META-INF/container.xml", deflated).unwrap();
    zip.write_all(
        br#"<?xml version="1.0"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>"#,
    )
    .unwrap();

    zip.start_file("OEBPS/content.opf", deflated).unwrap();
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:title>Test</dc:title>
    <dc:language>en</dc:language>
  </metadata>
  <manifest/><spine/>
</package>"#,
    )
    .unwrap();

    zip.start_file("OEBPS/chapter1.xhtml", deflated).unwrap();
    zip.write_all(b"<html><body>No encoding declaration</body></html>")
        .unwrap();

    zip.finish().unwrap().into_inner()
}

#[test]
fn cli_processes_epub_file() {
    let epub_data = build_test_epub();
    let mut tmpfile = NamedTempFile::with_suffix(".epub").unwrap();
    tmpfile.write_all(&epub_data).unwrap();
    tmpfile.flush().unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_kindle-file-fix"))
        .arg(tmpfile.path())
        .arg("--keep-name")
        .output()
        .expect("failed to execute");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "CLI failed: {}", stdout);
    assert!(stdout.contains("[FIXED]") || stdout.contains("[OK]"));
}

#[test]
fn cli_dry_run_does_not_write() {
    let epub_data = build_test_epub();
    let mut tmpfile = NamedTempFile::with_suffix(".epub").unwrap();
    tmpfile.write_all(&epub_data).unwrap();
    tmpfile.flush().unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_kindle-file-fix"))
        .arg(tmpfile.path())
        .arg("--dry-run")
        .output()
        .expect("failed to execute");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    assert!(!stdout.contains("Saved:"));
}

#[test]
fn cli_shows_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_kindle-file-fix"))
        .arg("--help")
        .output()
        .expect("failed to execute");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("kindle-file-fix"));
    assert!(stdout.contains("--language"));
    assert!(stdout.contains("--dry-run"));
}

#[test]
fn cli_fails_on_nonexistent_file() {
    let output = Command::new(env!("CARGO_BIN_EXE_kindle-file-fix"))
        .arg("/tmp/nonexistent-file-12345.epub")
        .output()
        .expect("failed to execute");

    assert!(!output.status.success());
}
```

**Step 2: Add tempfile and zip dev-dependencies to CLI Cargo.toml**

Add to `crates/kindle-fix-cli/Cargo.toml`:

```toml
[dev-dependencies]
tempfile = "3"
zip = "2"
```

**Step 3: Run tests to verify they pass**

Run: `cargo test --package kindle-file-fix`
Expected: All CLI tests PASS.

**Step 4: Commit**

```bash
git add -A
git commit -m "test(cli): add CLI integration tests"
```

---

## Task 13: README and Project Polish

**Files:**
- Modify: `README.md`

**Step 1: Write README**

```markdown
# kindle-file-fix

A cross-platform CLI tool (and upcoming GUI app) that fixes ebook files for Amazon Kindle compatibility.

## The Problem

Amazon's Send-to-Kindle service has several compatibility issues with EPUB files:
- Assumes ISO-8859-1 encoding when no XML encoding declaration is present
- Rejects hyperlinks that reference `<body>` element IDs
- Requires valid language metadata
- Fails on `<img>` tags with no `src` attribute

## Installation

### From GitHub Releases

Download the latest binary for your platform from [Releases](https://github.com/Klausdk1999/kindle-file-fix/releases).

### From Source

```bash
cargo install --path crates/kindle-fix-cli
```

## Usage

```bash
# Fix a single EPUB
kindle-file-fix book.epub

# Fix all EPUBs in a directory
kindle-file-fix ./my-books/

# Output to a specific directory
kindle-file-fix book.epub -o ./fixed/

# Set language explicitly (skip prompt)
kindle-file-fix book.epub --language en

# Preview fixes without writing files
kindle-file-fix book.epub --dry-run

# Keep original filename (no "(fixed)" prefix)
kindle-file-fix book.epub --keep-name
```

## What It Fixes

| Fix | Description |
|-----|-------------|
| **Encoding** | Adds `<?xml version="1.0" encoding="utf-8"?>` declaration to HTML/XHTML files missing it |
| **Body ID Links** | Removes `#body-id` hash references from hyperlinks that Kindle rejects |
| **Language Tags** | Validates and fixes `<dc:language>` metadata in OPF |
| **Stray Images** | Removes `<img>` tags with no `src` attribute |

## Supported Formats

| Format | Status |
|--------|--------|
| EPUB | Fully supported |
| MOBI | Planned |
| AZW3 | Planned |

## Building

```bash
# Build all crates
cargo build --release

# Run tests
cargo test

# Run with verbose logging
RUST_LOG=debug kindle-file-fix book.epub -v
```

## License

Apache-2.0. See [LICENSE](LICENSE) for details.

## Credits

Based on [kindle-epub-fix](https://github.com/innocenat/kindle-epub-fix) by innocenat (UNLICENSE).
```

**Step 2: Commit**

```bash
git add README.md
git commit -m "docs: add comprehensive README with usage and build instructions"
```

---

## Task 14: CI Workflow

**Files:**
- Create: `.github/workflows/ci.yml`

**Step 1: Write CI workflow**

```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always

jobs:
  check:
    name: Check & Test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - uses: Swatinem/rust-cache@v2

      - name: Check formatting
        run: cargo fmt --all -- --check

      - name: Clippy
        run: cargo clippy --all-targets -- -D warnings

      - name: Run tests
        run: cargo test --all

  build:
    name: Build (${{ matrix.target }})
    needs: check
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        include:
          - target: x86_64-unknown-linux-gnu
            os: ubuntu-latest
          - target: x86_64-apple-darwin
            os: macos-latest
          - target: aarch64-apple-darwin
            os: macos-latest
          - target: x86_64-pc-windows-msvc
            os: windows-latest

    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - uses: Swatinem/rust-cache@v2

      - name: Build CLI
        run: cargo build --release --package kindle-file-fix --target ${{ matrix.target }}

      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: kindle-file-fix-${{ matrix.target }}
          path: |
            target/${{ matrix.target }}/release/kindle-file-fix
            target/${{ matrix.target }}/release/kindle-file-fix.exe
```

**Step 2: Commit**

```bash
git add .github/workflows/ci.yml
git commit -m "ci: add GitHub Actions workflow for check, test, and build"
```

---

## Task 15: Release Workflow

**Files:**
- Create: `.github/workflows/release.yml`

**Step 1: Write release workflow**

```yaml
# .github/workflows/release.yml
name: Release

on:
  push:
    tags:
      - 'v*'

permissions:
  contents: write

env:
  CARGO_TERM_COLOR: always

jobs:
  build:
    name: Build (${{ matrix.target }})
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        include:
          - target: x86_64-unknown-linux-gnu
            os: ubuntu-latest
            artifact: kindle-file-fix
          - target: x86_64-apple-darwin
            os: macos-latest
            artifact: kindle-file-fix
          - target: aarch64-apple-darwin
            os: macos-latest
            artifact: kindle-file-fix
          - target: x86_64-pc-windows-msvc
            os: windows-latest
            artifact: kindle-file-fix.exe

    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - uses: Swatinem/rust-cache@v2

      - name: Build
        run: cargo build --release --package kindle-file-fix --target ${{ matrix.target }}

      - name: Package (Unix)
        if: matrix.os != 'windows-latest'
        run: |
          cd target/${{ matrix.target }}/release
          tar czf ../../../kindle-file-fix-${{ matrix.target }}.tar.gz ${{ matrix.artifact }}

      - name: Package (Windows)
        if: matrix.os == 'windows-latest'
        run: |
          cd target/${{ matrix.target }}/release
          7z a ../../../kindle-file-fix-${{ matrix.target }}.zip ${{ matrix.artifact }}

      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: kindle-file-fix-${{ matrix.target }}
          path: kindle-file-fix-${{ matrix.target }}.*

  release:
    name: Create Release
    needs: build
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Download artifacts
        uses: actions/download-artifact@v4
        with:
          merge-multiple: true

      - name: Create Release
        uses: softprops/action-gh-release@v2
        with:
          generate_release_notes: true
          files: |
            kindle-file-fix-*.tar.gz
            kindle-file-fix-*.zip
```

**Step 2: Commit**

```bash
git add .github/workflows/release.yml
git commit -m "ci: add release workflow for cross-platform binary distribution"
```

---

## Task 16: Tauri GUI Scaffolding

**Files:**
- Create: `crates/kindle-fix-gui/` (Tauri project)
- Create: `gui/index.html`
- Create: `gui/main.ts`
- Create: `gui/styles.css`
- Modify: `Cargo.toml` (add to workspace)

> **Note:** This task requires Tauri CLI (`cargo install tauri-cli`). If not available, scaffold manually.

**Step 1: Initialize Tauri project**

Run: `cargo install tauri-cli` (if not already installed)

Create the Tauri crate structure manually:

```toml
# crates/kindle-fix-gui/Cargo.toml
[package]
name = "kindle-fix-gui"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true

[dependencies]
kindle-fix-core = { path = "../kindle-fix-core" }
tauri = { version = "2", features = ["dialog"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"

[build-dependencies]
tauri-build = { version = "2", features = [] }
```

**Step 2: Create Tauri Rust backend**

```rust
// crates/kindle-fix-gui/src/main.rs
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::process_files,
            commands::get_supported_languages,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

```rust
// crates/kindle-fix-gui/src/commands.rs
use kindle_fix_core::formats::epub::fixes::language::SUPPORTED_LANGUAGES;
use kindle_fix_core::{process_file, FixOptions};
use serde::Serialize;
use std::fs;

#[derive(Serialize)]
pub struct GuiFixReport {
    pub filename: String,
    pub format: String,
    pub fixes: Vec<String>,
    pub warnings: Vec<String>,
    pub has_fixes: bool,
    pub error: Option<String>,
}

#[tauri::command]
pub fn process_files(
    paths: Vec<String>,
    language: Option<String>,
    keep_name: bool,
) -> Vec<GuiFixReport> {
    let options = FixOptions {
        language,
        keep_name,
        dry_run: false,
    };

    paths
        .iter()
        .map(|path| {
            let filename = std::path::Path::new(path)
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            match fs::read(path) {
                Ok(data) => match process_file(&data, &filename, &options) {
                    Ok(output) => {
                        // Write the fixed file
                        let output_path = if keep_name {
                            path.clone()
                        } else {
                            let p = std::path::Path::new(path);
                            let parent = p.parent().unwrap_or(std::path::Path::new("."));
                            parent
                                .join(format!("(fixed) {}", filename))
                                .to_string_lossy()
                                .to_string()
                        };

                        if !output.data.is_empty() {
                            fs::write(&output_path, &output.data).ok();
                        }

                        GuiFixReport {
                            filename,
                            format: output.report.format.to_string(),
                            fixes: output
                                .report
                                .fixes_applied
                                .iter()
                                .map(|f| f.details.clone())
                                .collect(),
                            warnings: output.report.warnings,
                            has_fixes: output.report.has_fixes(),
                            error: None,
                        }
                    }
                    Err(e) => GuiFixReport {
                        filename,
                        format: "Unknown".into(),
                        fixes: vec![],
                        warnings: vec![],
                        has_fixes: false,
                        error: Some(e.to_string()),
                    },
                },
                Err(e) => GuiFixReport {
                    filename,
                    format: "Unknown".into(),
                    fixes: vec![],
                    warnings: vec![],
                    has_fixes: false,
                    error: Some(format!("Could not read file: {}", e)),
                },
            }
        })
        .collect()
}

#[tauri::command]
pub fn get_supported_languages() -> Vec<String> {
    SUPPORTED_LANGUAGES.iter().map(|s| s.to_string()).collect()
}
```

**Step 3: Create frontend**

```html
<!-- gui/index.html -->
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Kindle File Fix</title>
    <link rel="stylesheet" href="styles.css">
</head>
<body>
    <main>
        <h1>Kindle File Fix</h1>
        <p>Drop your ebook files here or click to select.</p>

        <div id="dropzone" class="dropzone">
            <p>Drag & drop EPUB files here</p>
            <button id="selectBtn">Select Files</button>
        </div>

        <div id="options" class="options">
            <label>
                <input type="checkbox" id="keepName">
                Keep original filename
            </label>
        </div>

        <div id="results" class="results"></div>
    </main>
    <script type="module" src="main.ts"></script>
</body>
</html>
```

```css
/* gui/styles.css */
:root {
    --bg: #1a1a2e;
    --surface: #16213e;
    --primary: #0f3460;
    --accent: #e94560;
    --text: #eee;
    --text-muted: #999;
    --success: #4caf50;
    --warning: #ff9800;
    --error: #f44336;
}

@media (prefers-color-scheme: light) {
    :root {
        --bg: #f5f5f5;
        --surface: #fff;
        --primary: #1976d2;
        --accent: #e94560;
        --text: #333;
        --text-muted: #666;
    }
}

* { margin: 0; padding: 0; box-sizing: border-box; }

body {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    background: var(--bg);
    color: var(--text);
    min-height: 100vh;
}

main {
    max-width: 600px;
    margin: 0 auto;
    padding: 2rem;
}

h1 {
    margin-bottom: 0.5rem;
    font-size: 1.5rem;
}

.dropzone {
    border: 2px dashed var(--text-muted);
    border-radius: 12px;
    padding: 3rem 2rem;
    text-align: center;
    margin: 1.5rem 0;
    cursor: pointer;
    transition: border-color 0.2s;
}

.dropzone:hover, .dropzone.dragover {
    border-color: var(--accent);
}

.dropzone button {
    margin-top: 1rem;
    padding: 0.5rem 1.5rem;
    background: var(--primary);
    color: white;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.9rem;
}

.options {
    margin: 1rem 0;
}

.options label {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.9rem;
    color: var(--text-muted);
}

.results {
    margin-top: 1.5rem;
}

.result-item {
    background: var(--surface);
    border-radius: 8px;
    padding: 1rem;
    margin-bottom: 1rem;
}

.result-item h3 {
    font-size: 1rem;
    margin-bottom: 0.5rem;
}

.result-item .fix { color: var(--success); }
.result-item .warning { color: var(--warning); }
.result-item .error { color: var(--error); }
.result-item .ok { color: var(--primary); }

.result-item ul {
    list-style: none;
    padding-left: 1rem;
}

.result-item li::before {
    content: "• ";
}
```

```typescript
// gui/main.ts
const { invoke } = (window as any).__TAURI__.core;
const { open } = (window as any).__TAURI__.dialog;

const dropzone = document.getElementById("dropzone")!;
const selectBtn = document.getElementById("selectBtn")!;
const results = document.getElementById("results")!;
const keepName = document.getElementById("keepName") as HTMLInputElement;

interface FixReport {
    filename: string;
    format: string;
    fixes: string[];
    warnings: string[];
    has_fixes: boolean;
    error: string | null;
}

selectBtn.addEventListener("click", async () => {
    const selected = await open({
        multiple: true,
        filters: [{ name: "Ebooks", extensions: ["epub", "mobi", "azw3"] }],
    });

    if (selected) {
        const paths = Array.isArray(selected) ? selected : [selected];
        await processFiles(paths);
    }
});

// Drag and drop
dropzone.addEventListener("dragover", (e) => {
    e.preventDefault();
    dropzone.classList.add("dragover");
});

dropzone.addEventListener("dragleave", () => {
    dropzone.classList.remove("dragover");
});

dropzone.addEventListener("drop", async (e) => {
    e.preventDefault();
    dropzone.classList.remove("dragover");

    const files = e.dataTransfer?.files;
    if (files) {
        const paths: string[] = [];
        for (let i = 0; i < files.length; i++) {
            paths.push((files[i] as any).path);
        }
        await processFiles(paths);
    }
});

async function processFiles(paths: string[]) {
    results.innerHTML = "<p>Processing...</p>";

    const reports: FixReport[] = await invoke("process_files", {
        paths,
        language: null,
        keepName: keepName.checked,
    });

    results.innerHTML = "";

    for (const report of reports) {
        const div = document.createElement("div");
        div.className = "result-item";

        let statusHtml = "";
        if (report.error) {
            statusHtml = `<p class="error">${report.error}</p>`;
        } else if (report.fixes.length > 0) {
            statusHtml = `<ul>${report.fixes.map((f) => `<li class="fix">${f}</li>`).join("")}</ul>`;
        } else {
            statusHtml = `<p class="ok">No issues found.</p>`;
        }

        if (report.warnings.length > 0) {
            statusHtml += `<ul>${report.warnings.map((w) => `<li class="warning">${w}</li>`).join("")}</ul>`;
        }

        div.innerHTML = `<h3>${report.filename} <small>(${report.format})</small></h3>${statusHtml}`;
        results.appendChild(div);
    }
}
```

**Step 4: Create Tauri config and build file**

```json
// crates/kindle-fix-gui/tauri.conf.json
{
  "$schema": "https://raw.githubusercontent.com/tauri-apps/tauri/dev/crates/tauri-cli/config.schema.json",
  "productName": "Kindle File Fix",
  "version": "0.1.0",
  "identifier": "com.klausdk.kindle-file-fix",
  "build": {
    "frontendDist": "../../gui",
    "devUrl": "http://localhost:1420"
  },
  "app": {
    "windows": [
      {
        "title": "Kindle File Fix",
        "width": 600,
        "height": 500,
        "resizable": true,
        "fullscreen": false
      }
    ],
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/icon.ico"
    ]
  }
}
```

```rust
// crates/kindle-fix-gui/build.rs
fn main() {
    tauri_build::build()
}
```

**Step 5: Add GUI crate to workspace**

Add `"crates/kindle-fix-gui"` to workspace members in root `Cargo.toml`.

**Step 6: Commit**

```bash
git add -A
git commit -m "feat(gui): scaffold Tauri GUI app with drag-and-drop frontend"
```

---

## Task 17: Push to GitHub

**Step 1: Push all work**

```bash
git push -u origin main
```

---

## Summary of Commits

| # | Commit | Task |
|---|--------|------|
| 1 | `feat: scaffold Cargo workspace with core and CLI crates` | Task 1 |
| 2 | `feat(core): add types, error handling, and FixReport/FixOptions` | Task 2 |
| 3 | `feat(core): add EPUB reader with text/binary file separation` | Task 3 |
| 4 | `feat(core): add EPUB writer with mimetype-first ordering` | Task 4 |
| 5 | `feat(core): add UTF-8 encoding declaration fix` | Task 5 |
| 6 | `feat(core): add body ID link reference fix` | Task 6 |
| 7 | `feat(core): add language tag validation and fix` | Task 7 |
| 8 | `feat(core): add stray image tag removal fix` | Task 8 |
| 9 | `feat(core): add EpubFixer orchestration and MOBI/AZW3 stubs` | Task 9 |
| 10 | `feat(core): add process_file() public API with format auto-detection` | Task 10 |
| 11 | `feat(cli): add kindle-file-fix CLI with all options` | Task 11 |
| 12 | `test(cli): add CLI integration tests` | Task 12 |
| 13 | `docs: add comprehensive README with usage and build instructions` | Task 13 |
| 14 | `ci: add GitHub Actions workflow for check, test, and build` | Task 14 |
| 15 | `ci: add release workflow for cross-platform binary distribution` | Task 15 |
| 16 | `feat(gui): scaffold Tauri GUI app with drag-and-drop frontend` | Task 16 |
| 17 | Push to GitHub | Task 17 |
