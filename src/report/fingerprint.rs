use crate::report::Report;
use sha2::{Digest, Sha256};

/// Compute the SHA-256 fingerprint of a report's canonical JSON
/// body. The report_fingerprint field is excluded from the body
/// (see Report::canonical_json). The result is prefixed with
/// "sha256:" to identify the algorithm.
pub fn compute(report: &Report) -> String {
    let body = report.canonical_json();
    let mut hasher = Sha256::new();
    hasher.update(body.as_bytes());
    let digest = hasher.finalize();
    format!("sha256:{}", hex_encode(&digest))
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
