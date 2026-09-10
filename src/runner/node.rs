use crate::bundle::Bundle;
use crate::runner::{RunOutput, Runner};
use crate::spec::Spec;
use crate::witness::{Witness, WitnessResult};
use anyhow::{anyhow, Result};
use std::path::Path;

pub struct NodeRunner;

impl Runner for NodeRunner {
    fn name(&self) -> &'static str { "node" }
    fn prepare(&self, _dir: &Path, _spec: &Spec, _bundle: &Bundle, _w: &[Witness]) -> Result<()> {
        Err(anyhow!("node runner not yet implemented"))
    }
    fn invoke(&self, _dir: &Path, _args: &[String]) -> Result<RunOutput> {
        Err(anyhow!("node runner not yet implemented"))
    }
    fn parse(&self, _o: &RunOutput, _w: &[Witness]) -> Vec<WitnessResult> {
        Vec::new()
    }
}
