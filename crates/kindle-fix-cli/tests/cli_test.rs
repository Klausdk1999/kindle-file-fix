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
    assert!(
        output.status.success(),
        "CLI failed: {}\nstderr: {}",
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
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
        .arg("nonexistent-file-12345.epub")
        .output()
        .expect("failed to execute");

    assert!(!output.status.success());
}
