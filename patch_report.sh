#!/bin/bash
sed -i '/pub fn compute_fingerprint(&self) -> String {/i \    /// Canonical JSON including the report_fingerprint field.\n    /// Used for the --frozen output path.\n    pub fn canonical_json_with_fingerprint(&self) -> String {\n        serde_json::to_string(self)\n            .expect("Report must serialize to String")\n    }\n' src/report/mod.rs
