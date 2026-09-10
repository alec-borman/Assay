#[derive(Debug, Clone)]
pub struct Spec {
    pub name: String,
    pub targets: Vec<String>,
    pub runner: String,
    pub runner_args: Vec<String>,
    pub lang: String,
    pub fixtures: Vec<Fixture>,
    pub witnesses: Vec<crate::witness::Witness>,
}

#[derive(Debug, Clone)]
pub struct Fixture {
    pub name: String,
    pub body: String,
}
