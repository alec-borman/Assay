pub mod result;

pub use result::*;

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Kind {
    Hard,
    Soft(f64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Witness {
    pub name: String,
    pub kind: Kind,
    pub body: String,
}
