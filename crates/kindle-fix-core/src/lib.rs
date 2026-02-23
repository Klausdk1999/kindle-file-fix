//! Core library for fixing Kindle-incompatible ebook files.

pub mod error;
pub mod formats;
pub mod types;

pub use error::{KindleFixError, Result};
pub use types::{FileFormat, FixDescription, FixOptions, FixOutput, FixReport};

use formats::epub::EpubFixer;
use formats::mobi::MobiFixer;
use formats::FileFixer;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Process a file, auto-detecting its format and applying all relevant fixes.
pub fn process_file(data: &[u8], filename: &str, options: &FixOptions) -> Result<FixOutput> {
    if EpubFixer::detect(data) {
        let fixer = EpubFixer;
        let mut output = fixer.fix(data, options)?;
        output.report.filename = filename.to_string();
        Ok(output)
    } else if MobiFixer::detect(data) {
        let fixer = MobiFixer;
        let mut output = fixer.fix(data, options)?;
        output.report.filename = filename.to_string();
        Ok(output)
    } else {
        Err(KindleFixError::UnsupportedFormat(format!(
            "Could not detect format of '{}'",
            filename
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_set() {
        assert!(!version().is_empty());
    }

    #[test]
    fn fix_options_defaults() {
        let opts = FixOptions::default();
        assert!(opts.language.is_none());
        assert!(!opts.keep_name);
        assert!(!opts.dry_run);
    }

    #[test]
    fn fix_report_starts_empty() {
        let report = FixReport::new("test.epub".into(), FileFormat::Epub);
        assert_eq!(report.filename, "test.epub");
        assert!(report.fixes_applied.is_empty());
        assert!(report.warnings.is_empty());
    }

    #[test]
    fn file_format_display() {
        assert_eq!(format!("{}", FileFormat::Epub), "EPUB");
        assert_eq!(format!("{}", FileFormat::Mobi), "MOBI");
        assert_eq!(format!("{}", FileFormat::Azw3), "AZW3");
        assert_eq!(format!("{}", FileFormat::Unknown), "Unknown");
    }

    #[test]
    fn error_display() {
        let err = KindleFixError::InvalidEpub("missing mimetype".into());
        assert!(err.to_string().contains("missing mimetype"));
    }
}
