mod helpers;

use kindle_fix_core::{process_file, FileFormat, FixOptions};

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
