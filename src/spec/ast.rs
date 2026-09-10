#[derive(Debug, Clone)]
pub struct Spec {
    pub name: String,
    pub targets: Vec<String>,
    pub runner: String,
    pub runner_args: Vec<String>,
    pub lang: String,
    pub fixtures: Vec<Fixture>,
    pub witnesses: Vec<crate::witness::Witness>,
    pub objectives: Vec<Objective>,
}

#[derive(Debug, Clone)]
pub struct Fixture {
    pub name: String,
    pub body: String,
}

#[derive(Debug, Clone)]
pub struct Objective {
    pub kind: ObjectiveKind,
    pub name: String,
    pub target: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ObjectiveKind {
    Minimise,
    Maximise,
}
