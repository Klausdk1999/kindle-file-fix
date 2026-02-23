use kindle_fix_core::formats::epub::fixes::language::SUPPORTED_LANGUAGES;
use kindle_fix_core::{process_file, FixOptions};
use serde::Serialize;
use std::fs;

#[derive(Serialize)]
pub struct GuiFixReport {
    pub filename: String,
    pub format: String,
    pub fixes: Vec<String>,
    pub warnings: Vec<String>,
    pub has_fixes: bool,
    pub error: Option<String>,
}

#[tauri::command]
pub fn process_files(
    paths: Vec<String>,
    language: Option<String>,
    keep_name: bool,
) -> Vec<GuiFixReport> {
    let options = FixOptions {
        language,
        keep_name,
        dry_run: false,
    };

    paths
        .iter()
        .map(|path| {
            let filename = std::path::Path::new(path)
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            match fs::read(path) {
                Ok(data) => match process_file(&data, &filename, &options) {
                    Ok(output) => {
                        let output_path = if keep_name {
                            path.clone()
                        } else {
                            let p = std::path::Path::new(path);
                            let parent = p.parent().unwrap_or(std::path::Path::new("."));
                            parent
                                .join(format!("(fixed) {}", filename))
                                .to_string_lossy()
                                .to_string()
                        };

                        if !output.data.is_empty() {
                            fs::write(&output_path, &output.data).ok();
                        }

                        let has_fixes = output.report.has_fixes();
                        let format = output.report.format.to_string();
                        let fixes: Vec<String> = output
                            .report
                            .fixes_applied
                            .iter()
                            .map(|f| f.details.clone())
                            .collect();
                        let warnings = output.report.warnings;

                        GuiFixReport {
                            filename,
                            format,
                            fixes,
                            warnings,
                            has_fixes,
                            error: None,
                        }
                    }
                    Err(e) => GuiFixReport {
                        filename,
                        format: "Unknown".into(),
                        fixes: vec![],
                        warnings: vec![],
                        has_fixes: false,
                        error: Some(e.to_string()),
                    },
                },
                Err(e) => GuiFixReport {
                    filename,
                    format: "Unknown".into(),
                    fixes: vec![],
                    warnings: vec![],
                    has_fixes: false,
                    error: Some(format!("Could not read file: {}", e)),
                },
            }
        })
        .collect()
}

#[tauri::command]
pub fn get_supported_languages() -> Vec<String> {
    SUPPORTED_LANGUAGES.iter().map(|s| s.to_string()).collect()
}
