use std::io::{Cursor, Write};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

pub fn build_epub(files: &[(&str, &str)]) -> Vec<u8> {
    let buf = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(buf);

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

pub const CONTAINER_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>"#;

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
