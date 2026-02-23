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
