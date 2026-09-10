pub mod result;

pub use result::*;

#[derive(Debug, Clone)]
pub struct Witness {
    pub name: String,
    pub language: String,
    pub expression: String,
    pub mode: crate::spec::WitnessMode,
}
