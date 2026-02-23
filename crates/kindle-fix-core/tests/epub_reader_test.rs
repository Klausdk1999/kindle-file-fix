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
    use std::io::{Cursor, Write};
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    let buf = {
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
        zip.write_all(&[0x89, 0x50, 0x4E, 0x47]).unwrap();
        zip.finish().unwrap().into_inner()
    };

    let reader = EpubReader::from_bytes(&buf).unwrap();
    assert!(reader.text_files().contains_key("OEBPS/chapter1.xhtml"));
    assert!(reader.binary_files().contains_key("OEBPS/image.png"));
    assert!(!reader.text_files().contains_key("OEBPS/image.png"));
}
