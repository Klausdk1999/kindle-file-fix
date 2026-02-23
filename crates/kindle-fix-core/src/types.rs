use std::fmt;

#[derive(Debug, Clone, Default)]
pub struct FixOptions {
    pub language: Option<String>,
    pub keep_name: bool,
    pub dry_run: bool,
}

#[derive(Debug, Clone)]
pub struct FixReport {
    pub filename: String,
    pub format: FileFormat,
    pub fixes_applied: Vec<FixDescription>,
    pub warnings: Vec<String>,
}

impl FixReport {
    pub fn new(filename: String, format: FileFormat) -> Self {
        Self {
            filename,
            format,
            fixes_applied: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn has_fixes(&self) -> bool {
        !self.fixes_applied.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct FixDescription {
    pub name: String,
    pub details: String,
    pub files_affected: usize,
}

#[derive(Debug, Clone)]
pub struct FixOutput {
    pub data: Vec<u8>,
    pub report: FixReport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileFormat {
    Epub,
    Mobi,
    Azw3,
    Unknown,
}

impl fmt::Display for FileFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileFormat::Epub => write!(f, "EPUB"),
            FileFormat::Mobi => write!(f, "MOBI"),
            FileFormat::Azw3 => write!(f, "AZW3"),
            FileFormat::Unknown => write!(f, "Unknown"),
        }
    }
}
