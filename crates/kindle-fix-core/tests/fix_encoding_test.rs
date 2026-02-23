mod helpers;

use std::collections::HashMap;
use kindle_fix_core::formats::epub::fixes::encoding::fix_encoding;

#[test]
fn adds_encoding_to_xhtml_without_declaration() {
    let mut files = HashMap::new();
    files.insert(
        "chapter1.xhtml".to_string(),
        "<html><body>Hello</body></html>".to_string(),
    );
    let fixes = fix_encoding(&mut files);
    assert_eq!(fixes.len(), 1);
    assert!(files["chapter1.xhtml"].starts_with("<?xml version=\"1.0\" encoding=\"utf-8\"?>"));
}

#[test]
fn skips_files_with_existing_encoding() {
    let mut files = HashMap::new();
    files.insert(
        "chapter1.xhtml".to_string(),
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<html><body>Hello</body></html>".to_string(),
    );
    let fixes = fix_encoding(&mut files);
    assert!(fixes.is_empty());
}

#[test]
fn handles_single_quote_encoding() {
    let mut files = HashMap::new();
    files.insert(
        "chapter1.xhtml".to_string(),
        "<?xml version='1.0' encoding='utf-8'?>\n<html/>".to_string(),
    );
    let fixes = fix_encoding(&mut files);
    assert!(fixes.is_empty());
}

#[test]
fn only_processes_html_and_xhtml() {
    let mut files = HashMap::new();
    files.insert("style.css".to_string(), "body { color: red }".to_string());
    files.insert("content.opf".to_string(), "<package/>".to_string());
    files.insert("chapter.xhtml".to_string(), "<html/>".to_string());

    let fixes = fix_encoding(&mut files);
    assert_eq!(fixes.len(), 1);
    assert_eq!(files["style.css"], "body { color: red }");
    assert_eq!(files["content.opf"], "<package/>");
}

#[test]
fn handles_leading_whitespace() {
    let mut files = HashMap::new();
    files.insert(
        "chapter1.xhtml".to_string(),
        "  \n  <html><body>Hello</body></html>".to_string(),
    );
    let fixes = fix_encoding(&mut files);
    assert_eq!(fixes.len(), 1);
    assert!(files["chapter1.xhtml"].starts_with("<?xml version=\"1.0\" encoding=\"utf-8\"?>"));
}

#[test]
fn handles_htm_extension() {
    let mut files = HashMap::new();
    files.insert("chapter1.htm".to_string(), "<html/>".to_string());
    let fixes = fix_encoding(&mut files);
    assert_eq!(fixes.len(), 1);
}

#[test]
fn handles_html_extension() {
    let mut files = HashMap::new();
    files.insert("chapter1.html".to_string(), "<html/>".to_string());
    let fixes = fix_encoding(&mut files);
    assert_eq!(fixes.len(), 1);
}
