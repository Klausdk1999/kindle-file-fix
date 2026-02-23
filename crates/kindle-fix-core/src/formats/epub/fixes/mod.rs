pub mod body_id;
pub mod encoding;

/// Check if a filename has an HTML/XHTML extension.
pub(crate) fn is_html_file(filename: &str) -> bool {
    let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
    matches!(ext.as_str(), "html" | "xhtml" | "htm")
}
