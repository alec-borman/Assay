use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
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

#[derive(Debug, Clone, Serialize)]
pub struct Fixture {
    pub name: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Objective {
    pub kind: ObjectiveKind,
    pub name: String,
    pub target: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum ObjectiveKind {
    Minimise,
    Maximise,
}

impl Spec {
    /// Canonical JSON representation. Keys sorted, no
    /// insignificant whitespace. Input to the fingerprint.
    pub fn canonical_json(&self) -> String {
        let value = serde_json::to_value(self)
            .expect("Spec must be serializable to Value");
        serde_json::to_string(&value)
            .expect("Value must be serializable to String")
    }

    /// SHA-256 over canonical_json, prefixed with "sha256:".
    pub fn fingerprint(&self) -> String {
        use sha2::{Digest, Sha256};
        let body = self.canonical_json();
        let mut hasher = Sha256::new();
        hasher.update(body.as_bytes());
        let digest = hasher.finalize();
        format!("sha256:{}", hex_encode(&digest))
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0f) as usize] as char);
    }
    out
}
