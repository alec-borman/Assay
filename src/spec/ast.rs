#[derive(Debug, Clone)]
pub struct Spec {
    pub name: String,
    pub targets: Vec<String>,
    pub runner: String,
    pub runner_args: Vec<String>,
    pub lang: String,
    pub fixtures: Vec<Fixture>,
    pub witnesses: Vec<WitnessDecl>,
}

#[derive(Debug, Clone)]
pub struct Fixture {
    pub name: String,
    pub body: String,
}

#[derive(Debug, Clone)]
pub enum WitnessMode {
    Hard,
    Soft { weight: f64 },
}

#[derive(Debug, Clone)]
pub struct WitnessDecl {
    pub name: String,
    pub mode: WitnessMode,
    pub body: String,
}
