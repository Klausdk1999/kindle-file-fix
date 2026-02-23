use std::collections::HashMap;

use regex::Regex;

use super::is_html_file;

/// Fix body ID link references that Kindle rejects as unresolved hyperlinks.
pub fn fix_body_id_links(files: &mut HashMap<String, String>) -> Vec<String> {
    let body_id_regex =
        Regex::new(r#"<body\b[^>]*\bid\s*=\s*["']([^"']+)["'][^>]*>"#).expect("valid regex");

    let mut body_id_map: Vec<(String, String)> = Vec::new();

    let filenames: Vec<String> = files.keys().cloned().collect();
    for filename in &filenames {
        if !is_html_file(filename) {
            continue;
        }

        let content = &files[filename];
        if let Some(caps) = body_id_regex.captures(content) {
            let body_id = &caps[1];
            let basename = filename.rsplit('/').next().unwrap_or(filename);
            let src = format!("{}#{}", basename, body_id);
            body_id_map.push((src, basename.to_string()));
        }
    }

    if body_id_map.is_empty() {
        return Vec::new();
    }

    let mut fixes = Vec::new();

    for filename in &filenames {
        let content = files.get(filename).unwrap().clone();
        let mut modified = content.clone();

        for (src, target) in &body_id_map {
            if modified.contains(src.as_str()) {
                modified = modified.replace(src.as_str(), target.as_str());
                fixes.push(format!(
                    "Replaced link target {} with {} in {}",
                    src, target, filename
                ));
            }
        }

        if modified != content {
            files.insert(filename.clone(), modified);
        }
    }

    fixes
}
