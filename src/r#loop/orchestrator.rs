pub struct Orchestrator {
    pub max_iterations: usize,
    pub convergence_k: usize,
}

impl Orchestrator {
    pub fn new() -> Self {
        Self {
            max_iterations: 50,
            convergence_k: 3,
        }
    }

    pub fn run(&self) -> anyhow::Result<()> {
        Ok(())
    }
}
