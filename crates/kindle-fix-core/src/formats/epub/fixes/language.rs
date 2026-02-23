use std::collections::HashMap;

use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::{Reader, Writer};

pub const SUPPORTED_LANGUAGES: &[&str] = &[
    // ISO 639-1
    "af", "gsw", "ar", "eu", "nb", "br", "ca", "zh", "kw", "co", "da", "nl", "stq", "en", "fi",
    "fr", "fy", "gl", "de", "gu", "hi", "is", "ga", "it", "ja", "lb", "mr", "ml", "gv", "frr",
    "nn", "pl", "pt", "oc", "rm", "sco", "gd", "es", "sv", "ta", "cy",
    // ISO 639-2
    "afr", "ara", "eus", "baq", "nob", "bre", "cat", "zho", "chi", "cor", "cos", "dan", "nld",
    "dut", "eng", "fin", "fra", "fre", "fry", "glg", "deu", "ger", "guj", "hin", "isl", "ice",
    "gle", "ita", "jpn", "ltz", "mar", "mal", "glv", "nor", "nno", "por", "oci", "roh", "gla",
    "spa", "swe", "tam", "cym", "wel",
];

#[derive(Debug)]
pub enum LanguageFixResult {
    Valid(String),
    Added(String),
    Changed { from: String, to: String },
    Unsupported(String),
    Error(String),
}

fn simplify_language(lang: &str) -> String {
    lang.split('-').next().unwrap_or(lang).to_lowercase()
}

fn is_supported(lang: &str) -> bool {
    let simplified = simplify_language(lang);
    SUPPORTED_LANGUAGES.contains(&simplified.as_str())
}

fn find_opf_path(container_xml: &str) -> Option<String> {
    let mut reader = Reader::from_str(container_xml);
    loop {
        match reader.read_event() {
            Ok(Event::Empty(ref e)) | Ok(Event::Start(ref e))
                if e.name().as_ref() == b"rootfile" =>
            {
                let mut media_type = None;
                let mut full_path = None;
                for attr in e.attributes().flatten() {
                    match attr.key.as_ref() {
                        b"media-type" => {
                            media_type =
                                Some(String::from_utf8_lossy(&attr.value).to_string());
                        }
                        b"full-path" => {
                            full_path =
                                Some(String::from_utf8_lossy(&attr.value).to_string());
                        }
                        _ => {}
                    }
                }
                if media_type.as_deref() == Some("application/oebps-package+xml") {
                    return full_path;
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
    }
    None
}

fn extract_language(opf: &str) -> Option<String> {
    let mut reader = Reader::from_str(opf);
    let mut in_language = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                let name = e.name();
                let local = name.as_ref();
                if local == b"dc:language"
                    || local.ends_with(b":language")
                    || local == b"language"
                {
                    in_language = true;
                }
            }
            Ok(Event::Text(ref e)) if in_language => {
                return Some(e.unescape().unwrap_or_default().trim().to_string());
            }
            Ok(Event::End(_)) if in_language => {
                in_language = false;
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
    }
    None
}

fn replace_language_in_opf(opf: &str, new_lang: &str) -> String {
    let mut reader = Reader::from_str(opf);
    let mut writer = Writer::new(Vec::new());
    let mut in_language = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                let name = e.name();
                let local = name.as_ref();
                if local == b"dc:language"
                    || local.ends_with(b":language")
                    || local == b"language"
                {
                    in_language = true;
                }
                writer.write_event(Event::Start(e.clone())).ok();
            }
            Ok(Event::Text(ref _e)) if in_language => {
                writer
                    .write_event(Event::Text(BytesText::new(new_lang)))
                    .ok();
            }
            Ok(Event::End(ref e)) if in_language => {
                in_language = false;
                writer.write_event(Event::End(e.clone())).ok();
            }
            Ok(Event::Eof) => break,
            Ok(e) => {
                writer.write_event(e).ok();
            }
            Err(_) => break,
        }
    }

    String::from_utf8(writer.into_inner()).unwrap_or_else(|_| opf.to_string())
}

fn add_language_to_opf(opf: &str, lang: &str) -> Option<String> {
    let mut reader = Reader::from_str(opf);
    let mut writer = Writer::new(Vec::new());
    let mut added = false;

    loop {
        match reader.read_event() {
            Ok(Event::End(ref e)) if !added && e.name().as_ref() == b"metadata" => {
                writer
                    .write_event(Event::Start(BytesStart::new("dc:language")))
                    .ok();
                writer
                    .write_event(Event::Text(BytesText::new(lang)))
                    .ok();
                writer
                    .write_event(Event::End(BytesEnd::new("dc:language")))
                    .ok();
                writer.write_event(Event::End(e.clone())).ok();
                added = true;
            }
            Ok(Event::Eof) => break,
            Ok(e) => {
                writer.write_event(e).ok();
            }
            Err(_) => break,
        }
    }

    if added {
        Some(String::from_utf8(writer.into_inner()).unwrap_or_else(|_| opf.to_string()))
    } else {
        None
    }
}

pub fn fix_language(
    files: &mut HashMap<String, String>,
    language_override: Option<String>,
) -> LanguageFixResult {
    let container = match files.get("META-INF/container.xml") {
        Some(c) => c.clone(),
        None => return LanguageFixResult::Error("Missing META-INF/container.xml".into()),
    };

    let opf_path = match find_opf_path(&container) {
        Some(p) => p,
        None => {
            return LanguageFixResult::Error(
                "Could not find OPF file path in container.xml".into(),
            )
        }
    };

    let opf_content = match files.get(&opf_path) {
        Some(c) => c.clone(),
        None => return LanguageFixResult::Error(format!("OPF file not found: {}", opf_path)),
    };

    let current_language = extract_language(&opf_content);

    match current_language {
        None => {
            let lang = language_override.unwrap_or_else(|| "en".to_string());
            match add_language_to_opf(&opf_content, &lang) {
                Some(new_opf) => {
                    files.insert(opf_path, new_opf);
                    LanguageFixResult::Added(lang)
                }
                None => LanguageFixResult::Error("Failed to add language tag to OPF".into()),
            }
        }
        Some(lang) => {
            if is_supported(&lang) {
                LanguageFixResult::Valid(lang)
            } else if let Some(override_lang) = language_override {
                let new_opf = replace_language_in_opf(&opf_content, &override_lang);
                files.insert(opf_path, new_opf);
                LanguageFixResult::Changed {
                    from: lang,
                    to: override_lang,
                }
            } else {
                LanguageFixResult::Unsupported(lang)
            }
        }
    }
}
