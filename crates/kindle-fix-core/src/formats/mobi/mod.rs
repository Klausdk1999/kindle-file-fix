use crate::error::Result;
use crate::formats::FileFixer;
use crate::types::{FixOptions, FixOutput};
use crate::KindleFixError;

pub struct MobiFixer;

impl FileFixer for MobiFixer {
    fn detect(data: &[u8]) -> bool {
        data.len() > 68 && &data[60..68] == b"BOOKMOBI"
    }

    fn fix(&self, _data: &[u8], _options: &FixOptions) -> Result<FixOutput> {
        Err(KindleFixError::UnsupportedFormat(
            "MOBI support coming soon".into(),
        ))
    }
}
