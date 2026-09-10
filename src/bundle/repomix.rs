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
    let mut files = BTreeMap::new();

    let files_start = content
        .find("<files>")
        .ok_or_else(|| anyhow!("missing <files> block"))?
        + "<files>".len();
    let files_end = content[files_start..]
        .find("</files>")
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

        // The real closing </file> is the LAST one before the
        // next <file or the end of the files block. Any </file>
        // earlier in the slice is inside the file's content.
        let next_block = files_block[body_start..]
            .find("<file ")
            .map(|p| p + body_start)
            .or_else(|| {
                files_block[body_start..]
                    .find("</files>")
                    .map(|p| p + body_start)
            })
            .unwrap_or(files_block.len());

        let body_end = files_block[body_start..next_block]
            .rfind("</file>")
            .ok_or_else(|| anyhow!("missing </file> for path {}", path))?
            + body_start;

        let body = &files_block[body_start..body_end];
        let content_str = extract_cdata(body);

        files.insert(path, content_str);
        cursor = body_end + "</file>".len();
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
    Ok(value.to_string())
}

fn extract_cdata(body: &str) -> String {
    const OPEN: &str = "<![CDATA[";
    const CLOSE: &str = "]]>";
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
    result
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
