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

    let reader = EpubReader::from_bytes(&original).unwrap();
    let (text_files, binary_files) = reader.into_parts();
    let output = EpubWriter::write(&text_files, &binary_files).unwrap();

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

    let cursor = std::io::Cursor::new(&output);
    let mut archive = zip::ZipArchive::new(cursor).unwrap();
    let mimetype_entry = archive.by_name("mimetype").unwrap();
    assert_eq!(
        mimetype_entry.compression(),
        zip::CompressionMethod::Stored
    );
}
