pub mod azw3;
pub mod epub;
pub mod mobi;

use crate::error::Result;
use crate::types::{FixOptions, FixOutput};

/// Trait for format-specific file fixers.
pub trait FileFixer {
    /// Detect if the given data matches this format.
    fn detect(data: &[u8]) -> bool;

    /// Apply all fixes and return the fixed data with a report.
    fn fix(&self, data: &[u8], options: &FixOptions) -> Result<FixOutput>;
}
