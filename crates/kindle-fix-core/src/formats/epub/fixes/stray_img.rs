use std::collections::HashMap;

use regex::Regex;

use super::is_html_file;

/// Remove `<img>` tags that have no `src` attribute.
/// Returns a list of filenames where stray images were removed.
pub fn fix_stray_images(files: &mut HashMap<String, String>) -> Vec<String> {
    let img_regex = Regex::new(r#"<img\b([^>]*)/?>"#).expect("valid regex");
    let src_regex = Regex::new(r#"\bsrc\s*="#).expect("valid regex");

    let mut fixed = Vec::new();

    let filenames: Vec<String> = files.keys().cloned().collect();
    for filename in filenames {
        if !is_html_file(&filename) {
            continue;
        }

        let content = files[&filename].clone();
        let mut removed_count = 0;

        let new_content = img_regex.replace_all(&content, |caps: &regex::Captures| {
            let attrs = &caps[1];
            if src_regex.is_match(attrs) {
                caps[0].to_string()
            } else {
                removed_count += 1;
                String::new()
            }
        });

        if removed_count > 0 {
            files.insert(filename.clone(), new_content.to_string());
            fixed.push(filename);
        }
    }

    fixed
}
