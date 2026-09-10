use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Directive {
    pub id: String,
    pub target: Vec<String>,
    pub intent: String,
    pub constraints: Vec<String>,
    pub rationale: String,
    pub priority: String,
    pub expected_outcome: ExpectedOutcome,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExpectedOutcome {
    pub witnesses_that_should_flip: Vec<String>,
    pub objective_target: f64,
}
