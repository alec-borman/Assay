// src/bundle/repomix.rs

use crate::bundle::Bundle;
use anyhow::{anyhow, Result};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

pub fn parse(content: &str) -> Result<Bundle> {
    let metadata = extract_summary(content);
    let files = extract_files(content)?;
    let fingerprint = compute_fingerprint(&files);
    Ok(Bundle {
        files,
        metadata,
        fingerprint,
    })
}

fn extract_summary(content: &str) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    if let Some(start) = content.find("<file_summary>") {
        let after = &content[start + "<file_summary>".len()..];
        if let Some(end) = after.find("</file_summary>") {
            let raw = after[..end].trim().to_string();
            map.insert("raw".to_string(), raw);
        }
    }
    map
}

fn extract_files(content: &str) -> Result<BTreeMap<String, String>> {
    const CDATA_OPEN: &str = "<![CDATA[";
    const CDATA_CLOSE: &str = "]]>";

    let mut files = BTreeMap::new();

    let files_start = content
        .find("<files>")
        .ok_or_else(|| anyhow!("missing <files> block"))?
        + "<files>".len();
    let files_end = content[files_start..]
        .rfind("</files>")
        .ok_or_else(|| anyhow!("missing </files> terminator"))?
        + files_start;
    let files_block = &content[files_start..files_end];

    let mut cursor = 0;
    while let Some(pos) = files_block[cursor..].find("<file ") {
        let tag_start = cursor + pos;
        let after_tag = tag_start + "<file ".len();

        let tag_end = files_block[after_tag..]
            .find('>')
            .ok_or_else(|| anyhow!("unterminated <file> opening tag"))?
            + after_tag;

        let attr_str = &files_block[after_tag..tag_end];
        let path = extract_path_attr(attr_str)?;

        let body_start = tag_end + 1;

        // Determine if this file entry is wrapped in CDATA
        let rest_from_body = &files_block[body_start..];
        let has_cdata = rest_from_body.trim_start().starts_with(CDATA_OPEN);

        let (content_str, next_cursor) = if has_cdata {
            let mut scan = body_start;
            let after_cdata = loop {
                let open_pos = files_block[scan..]
                    .find(CDATA_OPEN)
                    .ok_or_else(|| anyhow!("missing CDATA for path {}", path))?
                    + scan;
                let content_start = open_pos + CDATA_OPEN.len();
                let close_pos = files_block[content_start..]
                    .find(CDATA_CLOSE)
                    .ok_or_else(|| anyhow!("missing CDATA close for path {}", path))?
                    + content_start;
                let after_close = close_pos + CDATA_CLOSE.len();

                if files_block[after_close..].starts_with(CDATA_OPEN) {
                    scan = after_close;
                    continue;
                }
                break after_close;
            };

            let terminator = files_block[after_cdata..]
                .find("</file>")
                .ok_or_else(|| anyhow!("missing </file> for path {}", path))?
                + after_cdata;

            let body = &files_block[body_start..terminator];
            (extract_cdata(body), terminator + "</file>".len())
        } else {
            let mut search_pos = body_start;
            let terminator = loop {
                let rel = files_block[search_pos..]
                    .find("</file>")
                    .ok_or_else(|| anyhow!("missing </file> for path {}", path))?;
                let term = search_pos + rel;
                let after = &files_block[term + "</file>".len()..];
                let after_trimmed = after.trim_start();

                // 1. The genuine closing </files> tag has only whitespace following it
                if after_trimmed.is_empty() {
                    break term;
                }
                if let Some(rest) = after_trimmed.strip_prefix("</files>") {
                    if rest.trim().is_empty() {
                        break term;
                    }
                }

                // 2. A genuine next file tag is on its own line ending with a newline
                if after_trimmed.starts_with("<file ") {
                    if let Some(gt) = after_trimmed.find('>') {
                        let candidate_attr = &after_trimmed["<file ".len()..gt];
                        if extract_path_attr(candidate_attr).is_ok() {
                            let after_gt = &after_trimmed[gt + 1..];
                            if after_gt.starts_with('\n') || after_gt.starts_with("\r\n") {
                                break term;
                            }
                        }
                    }
                }

                search_pos = term + "</file>".len();
            };

            let body = &files_block[body_start..terminator];
            (clean_plain_body(body), terminator + "</file>".len())
        };

        files.insert(path, content_str);
        cursor = next_cursor;
    }

    Ok(files)
}

fn extract_path_attr(attr: &str) -> Result<String> {
    let attr = attr.trim();
    let eq = attr
        .find('=')
        .ok_or_else(|| anyhow!("malformed file attribute: {}", attr))?;
    let value = attr[eq + 1..].trim();
    let value = value
        .strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .ok_or_else(|| anyhow!("expected quoted path in: {}", attr))?;

    // Normalize Windows backslashes to forward slashes for cross-platform determinism
    Ok(value.replace('\\', "/"))
}

fn extract_cdata(body: &str) -> String {
    const OPEN: &str = "<![CDATA[";
    const CLOSE: &str = "]]>";

    if !body.contains(OPEN) {
        return body.trim().to_string();
    }

    let mut result = String::new();
    let mut cursor = 0;

    while let Some(open_rel) = body[cursor..].find(OPEN) {
        let content_start = cursor + open_rel + OPEN.len();
        let close_rel = match body[content_start..].find(CLOSE) {
            Some(p) => p,
            None => break,
        };
        let content_end = content_start + close_rel;
        result.push_str(&body[content_start..content_end]);
        cursor = content_end + CLOSE.len();
    }

    clean_plain_body(&result)
}

fn clean_plain_body(body: &str) -> String {
    let mut s = body;
    if let Some(stripped) = s.strip_prefix("\r\n") {
        s = stripped;
    } else if let Some(stripped) = s.strip_prefix('\n') {
        s = stripped;
    }

    if let Some(stripped) = s.strip_suffix("\r\n") {
        s = stripped;
    } else if let Some(stripped) = s.strip_suffix('\n') {
        s = stripped;
    }

    s.to_string()
}

fn compute_fingerprint(files: &BTreeMap<String, String>) -> String {
    let mut hasher = Sha256::new();
    for (path, content) in files {
        hasher.update(path.as_bytes());
        hasher.update(&[0u8]);
        hasher.update(content.as_bytes());
        hasher.update(&[0u8]);
    }
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
