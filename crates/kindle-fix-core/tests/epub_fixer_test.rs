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
    let output = fixer.fix(&epub, &FixOptions::default()).unwrap();

    assert!(!output.report.fixes_applied.is_empty());
    assert!(output
        .report
        .fixes_applied
        .iter()
        .any(|f| f.name == "encoding"));
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
    let output = fixer.fix(&epub, &FixOptions::default()).unwrap();
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
    let output = fixer.fix(&epub, &FixOptions::default()).unwrap();

    let reader =
        kindle_fix_core::formats::epub::reader::EpubReader::from_bytes(&output.data).unwrap();
    assert!(reader.text_files().contains_key("mimetype"));
}
