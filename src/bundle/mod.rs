pub mod repomix;
pub mod markdown;
pub mod json;

use std::collections::BTreeMap;

pub struct Bundle {
    pub files: BTreeMap<String, String>,
    pub metadata: BTreeMap<String, String>,
    pub fingerprint: String,
}

impl Bundle {
    pub fn empty() -> Self {
        Self {
            files: BTreeMap::new(),
            metadata: BTreeMap::new(),
            fingerprint: String::new(),
        }
    }
}
