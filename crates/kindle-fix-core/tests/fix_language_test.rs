mod helpers;

use std::collections::HashMap;
use kindle_fix_core::formats::epub::fixes::language::{
    fix_language, LanguageFixResult, SUPPORTED_LANGUAGES,
};

#[test]
fn detects_missing_language() {
    let mut files = HashMap::new();
    files.insert(
        "META-INF/container.xml".to_string(),
        helpers::CONTAINER_XML.to_string(),
    );
    files.insert(
        "OEBPS/content.opf".to_string(),
        helpers::opf_without_language(),
    );

    let result = fix_language(&mut files, Some("en".to_string()));
    match result {
        LanguageFixResult::Added(lang) => assert_eq!(lang, "en"),
        other => panic!("Expected Added, got {:?}", other),
    }
    assert!(files["OEBPS/content.opf"].contains("dc:language"));
}

#[test]
fn detects_valid_language() {
    let mut files = HashMap::new();
    files.insert(
        "META-INF/container.xml".to_string(),
        helpers::CONTAINER_XML.to_string(),
    );
    files.insert(
        "OEBPS/content.opf".to_string(),
        helpers::opf_with_language("en"),
    );

    let result = fix_language(&mut files, None);
    match result {
        LanguageFixResult::Valid(lang) => assert_eq!(lang, "en"),
        other => panic!("Expected Valid, got {:?}", other),
    }
}

#[test]
fn detects_unsupported_language_with_override() {
    let mut files = HashMap::new();
    files.insert(
        "META-INF/container.xml".to_string(),
        helpers::CONTAINER_XML.to_string(),
    );
    files.insert(
        "OEBPS/content.opf".to_string(),
        helpers::opf_with_language("xx"),
    );

    let result = fix_language(&mut files, Some("en".to_string()));
    match result {
        LanguageFixResult::Changed { from, to } => {
            assert_eq!(from, "xx");
            assert_eq!(to, "en");
        }
        other => panic!("Expected Changed, got {:?}", other),
    }
}

#[test]
fn detects_unsupported_language_without_override() {
    let mut files = HashMap::new();
    files.insert(
        "META-INF/container.xml".to_string(),
        helpers::CONTAINER_XML.to_string(),
    );
    files.insert(
        "OEBPS/content.opf".to_string(),
        helpers::opf_with_language("xx"),
    );

    let result = fix_language(&mut files, None);
    match result {
        LanguageFixResult::Unsupported(lang) => assert_eq!(lang, "xx"),
        other => panic!("Expected Unsupported, got {:?}", other),
    }
}

#[test]
fn handles_regional_language_codes() {
    let mut files = HashMap::new();
    files.insert(
        "META-INF/container.xml".to_string(),
        helpers::CONTAINER_XML.to_string(),
    );
    files.insert(
        "OEBPS/content.opf".to_string(),
        helpers::opf_with_language("en-US"),
    );

    let result = fix_language(&mut files, None);
    match result {
        LanguageFixResult::Valid(lang) => assert_eq!(lang, "en-US"),
        other => panic!("Expected Valid, got {:?}", other),
    }
}

#[test]
fn case_insensitive_language_check() {
    let mut files = HashMap::new();
    files.insert(
        "META-INF/container.xml".to_string(),
        helpers::CONTAINER_XML.to_string(),
    );
    files.insert(
        "OEBPS/content.opf".to_string(),
        helpers::opf_with_language("EN"),
    );

    let result = fix_language(&mut files, None);
    match result {
        LanguageFixResult::Valid(lang) => assert_eq!(lang, "EN"),
        other => panic!("Expected Valid, got {:?}", other),
    }
}

#[test]
fn returns_error_on_missing_container_xml() {
    let mut files = HashMap::new();
    files.insert(
        "OEBPS/content.opf".to_string(),
        helpers::opf_with_language("en"),
    );

    let result = fix_language(&mut files, None);
    assert!(matches!(result, LanguageFixResult::Error(_)));
}

#[test]
fn supported_languages_includes_common_codes() {
    assert!(SUPPORTED_LANGUAGES.contains(&"en"));
    assert!(SUPPORTED_LANGUAGES.contains(&"fr"));
    assert!(SUPPORTED_LANGUAGES.contains(&"de"));
    assert!(SUPPORTED_LANGUAGES.contains(&"ja"));
    assert!(SUPPORTED_LANGUAGES.contains(&"eng"));
    assert!(SUPPORTED_LANGUAGES.contains(&"fra"));
}
