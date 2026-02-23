mod helpers;

use std::collections::HashMap;
use kindle_fix_core::formats::epub::fixes::stray_img::fix_stray_images;

#[test]
fn removes_img_without_src() {
    let mut files = HashMap::new();
    files.insert(
        "chapter.xhtml".to_string(),
        r#"<?xml version="1.0" encoding="utf-8"?><html><body><p>Text</p><img/><p>More</p></body></html>"#.to_string(),
    );
    let fixes = fix_stray_images(&mut files);
    assert_eq!(fixes.len(), 1);
    assert!(!files["chapter.xhtml"].contains("<img"));
}

#[test]
fn keeps_img_with_src() {
    let mut files = HashMap::new();
    files.insert(
        "chapter.xhtml".to_string(),
        r#"<?xml version="1.0" encoding="utf-8"?><html><body><img src="image.png"/></body></html>"#.to_string(),
    );
    let fixes = fix_stray_images(&mut files);
    assert!(fixes.is_empty());
    assert!(files["chapter.xhtml"].contains(r#"src="image.png""#));
}

#[test]
fn handles_multiple_stray_images() {
    let mut files = HashMap::new();
    files.insert(
        "chapter.xhtml".to_string(),
        r#"<?xml version="1.0" encoding="utf-8"?><html><body><img/><img src="ok.png"/><img/></body></html>"#.to_string(),
    );
    let fixes = fix_stray_images(&mut files);
    assert_eq!(fixes.len(), 1);
    assert!(files["chapter.xhtml"].contains("ok.png"));
}

#[test]
fn only_processes_html_files() {
    let mut files = HashMap::new();
    files.insert("style.css".to_string(), "img { display: none }".to_string());
    files.insert(
        "chapter.xhtml".to_string(),
        r#"<?xml version="1.0" encoding="utf-8"?><html><body><img/></body></html>"#.to_string(),
    );
    let fixes = fix_stray_images(&mut files);
    assert_eq!(fixes.len(), 1);
    assert_eq!(files["style.css"], "img { display: none }");
}
