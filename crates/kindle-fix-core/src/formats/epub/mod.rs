pub mod fixes;
pub mod reader;
pub mod writer;

use std::io::{Cursor, Read};

use crate::error::Result;
use crate::formats::FileFixer;
use crate::types::{FileFormat, FixDescription, FixOptions, FixOutput, FixReport};

use self::fixes::body_id::fix_body_id_links;
use self::fixes::encoding::fix_encoding;
use self::fixes::language::{fix_language, LanguageFixResult};
use self::fixes::stray_img::fix_stray_images;
use self::reader::EpubReader;
use self::writer::EpubWriter;

pub struct EpubFixer;

impl FileFixer for EpubFixer {
    fn detect(data: &[u8]) -> bool {
        let cursor = Cursor::new(data);
        if let Ok(mut archive) = zip::ZipArchive::new(cursor) {
            if let Ok(mut entry) = archive.by_name("mimetype") {
                let mut content = String::new();
                if entry.read_to_string(&mut content).is_ok() {
                    return content.trim() == "application/epub+zip";
                }
            }
        }
        false
    }

    fn fix(&self, data: &[u8], options: &FixOptions) -> Result<FixOutput> {
        let reader = EpubReader::from_bytes(data)?;
        let (mut text_files, binary_files) = reader.into_parts();
        let mut report = FixReport::new(String::new(), FileFormat::Epub);

        // Fix 1: Body ID links
        let body_id_fixes = fix_body_id_links(&mut text_files);
        if !body_id_fixes.is_empty() {
            report.fixes_applied.push(FixDescription {
                name: "body_id".to_string(),
                details: format!("Removed {} body ID link reference(s)", body_id_fixes.len()),
                files_affected: body_id_fixes.len(),
            });
        }

        // Fix 2: Language
        let lang_result = fix_language(&mut text_files, options.language.clone());
        match &lang_result {
            LanguageFixResult::Added(lang) => {
                report.fixes_applied.push(FixDescription {
                    name: "language".to_string(),
                    details: format!("Added missing language tag: {}", lang),
                    files_affected: 1,
                });
            }
            LanguageFixResult::Changed { from, to } => {
                report.fixes_applied.push(FixDescription {
                    name: "language".to_string(),
                    details: format!("Changed language from {} to {}", from, to),
                    files_affected: 1,
                });
            }
            LanguageFixResult::Unsupported(lang) => {
                report.warnings.push(format!(
                    "Language '{}' is not supported by Kindle. Use --language to override.",
                    lang
                ));
            }
            LanguageFixResult::Error(msg) => {
                report.warnings.push(format!("Language check failed: {}", msg));
            }
            LanguageFixResult::Valid(_) => {}
        }

        // Fix 3: Stray images
        let stray_img_fixes = fix_stray_images(&mut text_files);
        if !stray_img_fixes.is_empty() {
            report.fixes_applied.push(FixDescription {
                name: "stray_img".to_string(),
                details: format!(
                    "Removed stray image tag(s) in {} file(s)",
                    stray_img_fixes.len()
                ),
                files_affected: stray_img_fixes.len(),
            });
        }

        // Fix 4: Encoding
        let encoding_fixes = fix_encoding(&mut text_files);
        if !encoding_fixes.is_empty() {
            report.fixes_applied.push(FixDescription {
                name: "encoding".to_string(),
                details: format!(
                    "Added UTF-8 encoding declaration to {} file(s)",
                    encoding_fixes.len()
                ),
                files_affected: encoding_fixes.len(),
            });
        }

        let output_data = if options.dry_run {
            Vec::new()
        } else {
            EpubWriter::write(&text_files, &binary_files)?
        };

        Ok(FixOutput {
            data: output_data,
            report,
        })
    }
}
