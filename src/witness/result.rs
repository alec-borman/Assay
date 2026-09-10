use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Pass,
    Fail,
    Skip,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WitnessResult {
    pub name: String,
    pub status: Status,
    pub detail: Option<String>,
    pub duration_ms: Option<u64>,
}
