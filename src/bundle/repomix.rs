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
    let mut files = BTreeMap::new();

    let files_start = content
        .find("<files>")
        .ok_or_else(|| anyhow!("missing <files> block"))?
        + "<files>".len();
        
    // Use rfind to get the true end of the <files> block, avoiding literal
    // "</files>" strings that might appear inside the source files themselves.
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

        // Find the matching </file>, but we MUST skip over any CDATA blocks
        // because literal strings like "</file>" might appear inside them.
        let mut search_cursor = body_start;
        let body_end = loop {
            let next_cdata = files_block[search_cursor..].find("<![CDATA[");
            let next_close = files_block[search_cursor..].find("</file>");

            match (next_cdata, next_close) {
                // If there's a CDATA block before the </file>, we must skip past it
                (Some(cdata_pos), Some(close_pos)) if cdata_pos < close_pos => {
                    let cdata_start = search_cursor + cdata_pos;
                    let cdata_close = files_block[cdata_start..]
                        .find("]]>")
                        .ok_or_else(|| anyhow!("unterminated CDATA for path {}", path))?;
                    search_cursor = cdata_start + cdata_close + "]]>".len();
                }
                // If there's a </file> and it comes before any CDATA (or there is no CDATA)
                (_, Some(close_pos)) => {
                    break search_cursor + close_pos;
                }
                // We ran out of </file> tags
                (_, None) => return Err(anyhow!("missing </file> for path {}", path)),
            }
        };

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
    
    // Stitch multiple CDATA blocks together (Repomix escapes `]]>` as `]]]]><![CDATA[>`)
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
    
    // Strip only the single leading and trailing newline inserted by Repomix around the content
    let mut final_result = result.as_str();
    if let Some(stripped) = final_result.strip_prefix("\r\n") {
        final_result = stripped;
    } else if let Some(stripped) = final_result.strip_prefix('\n') {
        final_result = stripped;
    }

    if let Some(stripped) = final_result.strip_suffix("\r\n") {
        final_result = stripped;
    } else if let Some(stripped) = final_result.strip_suffix('\n') {
        final_result = stripped;
    }
    
    final_result.to_string()
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
