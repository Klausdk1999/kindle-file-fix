use std::collections::HashMap;
use std::io::{Cursor, Write};

use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use crate::error::Result;

pub struct EpubWriter;

impl EpubWriter {
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
