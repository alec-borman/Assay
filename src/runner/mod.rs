pub mod rust;
pub mod python;
pub mod node;
pub mod shell;

use crate::spec::Spec;
use crate::witness::{Witness, WitnessResult};
use std::path::Path;
use anyhow::Result;

pub struct RunOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
    pub per_witness: Vec<WitnessResult>,
}

pub trait Runner {
    fn name(&self) -> &'static str;
    fn prepare(&self, dir: &Path, spec: &Spec, witnesses: &[Witness]) -> Result<()>;
    fn invoke(&self, dir: &Path, args: &[String]) -> Result<RunOutput>;
    fn parse(&self, output: &RunOutput, witnesses: &[Witness]) -> Vec<WitnessResult>;
}
