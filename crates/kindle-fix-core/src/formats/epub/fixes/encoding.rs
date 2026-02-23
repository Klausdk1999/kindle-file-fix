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
