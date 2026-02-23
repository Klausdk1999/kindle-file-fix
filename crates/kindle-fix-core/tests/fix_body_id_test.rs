mod helpers;

use std::collections::HashMap;
use kindle_fix_core::formats::epub::fixes::body_id::fix_body_id_links;

#[test]
fn replaces_body_id_references() {
    let mut files = HashMap::new();
    files.insert(
        "OEBPS/chapter1.xhtml".to_string(),
        r#"<html><body id="chapter1body">Content</body></html>"#.to_string(),
    );
    files.insert(
        "OEBPS/toc.ncx".to_string(),
        r#"<navPoint><content src="chapter1.xhtml#chapter1body"/></navPoint>"#.to_string(),
    );

    let fixes = fix_body_id_links(&mut files);
    assert_eq!(fixes.len(), 1);
    assert!(files["OEBPS/toc.ncx"].contains("chapter1.xhtml\""));
    assert!(!files["OEBPS/toc.ncx"].contains("#chapter1body"));
}

#[test]
fn ignores_non_body_id_hashes() {
    let mut files = HashMap::new();
    files.insert(
        "OEBPS/chapter1.xhtml".to_string(),
        r#"<html><body>Content <div id="section1">Section</div></body></html>"#.to_string(),
    );
    files.insert(
        "OEBPS/toc.ncx".to_string(),
        r#"<navPoint><content src="chapter1.xhtml#section1"/></navPoint>"#.to_string(),
    );

    let fixes = fix_body_id_links(&mut files);
    assert!(fixes.is_empty());
    assert!(files["OEBPS/toc.ncx"].contains("#section1"));
}

#[test]
fn skips_body_without_id() {
    let mut files = HashMap::new();
    files.insert(
        "OEBPS/chapter1.xhtml".to_string(),
        r#"<html><body>Content</body></html>"#.to_string(),
    );

    let fixes = fix_body_id_links(&mut files);
    assert!(fixes.is_empty());
}

#[test]
fn handles_multiple_files_with_body_ids() {
    let mut files = HashMap::new();
    files.insert(
        "OEBPS/ch1.xhtml".to_string(),
        r#"<html><body id="b1">Ch1</body></html>"#.to_string(),
    );
    files.insert(
        "OEBPS/ch2.xhtml".to_string(),
        r#"<html><body id="b2">Ch2</body></html>"#.to_string(),
    );
    files.insert(
        "OEBPS/toc.ncx".to_string(),
        r#"<a href="ch1.xhtml#b1"/><a href="ch2.xhtml#b2"/>"#.to_string(),
    );

    let fixes = fix_body_id_links(&mut files);
    assert_eq!(fixes.len(), 2);
    assert!(files["OEBPS/toc.ncx"].contains("ch1.xhtml\""));
    assert!(files["OEBPS/toc.ncx"].contains("ch2.xhtml\""));
}
