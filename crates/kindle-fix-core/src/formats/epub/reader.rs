use std::collections::HashMap;
use std::io::{Cursor, Read};

use crate::error::Result;

const TEXT_EXTENSIONS: &[&str] = &[
    "html", "xhtml", "htm", "xml", "svg", "css", "opf", "ncx",
];

const TEXT_FILENAMES: &[&str] = &["mimetype"];

pub struct EpubReader {
    text_files: HashMap<String, String>,
    binary_files: HashMap<String, Vec<u8>>,
}

impl EpubReader {
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        let cursor = Cursor::new(data);
        let mut archive = zip::ZipArchive::new(cursor)?;

        let mut text_files = HashMap::new();
        let mut binary_files = HashMap::new();

        for i in 0..archive.len() {
            let mut entry = archive.by_index(i)?;
            let name = entry.name().to_string();

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

    pub fn into_parts(self) -> (HashMap<String, String>, HashMap<String, Vec<u8>>) {
        (self.text_files, self.binary_files)
    }
}

fn is_text_file(filename: &str) -> bool {
    let basename = filename.rsplit('/').next().unwrap_or(filename);
    if TEXT_FILENAMES.contains(&basename) {
        return true;
    }

    if let Some(ext) = filename.rsplit('.').next() {
        TEXT_EXTENSIONS.contains(&ext.to_lowercase().as_str())
    } else {
        false
    }
}
