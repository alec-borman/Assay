use crate::spec::{Spec, Fixture};
use crate::witness::{Witness, Kind};
use anyhow::Result;

fn split_quoted(s: &str) -> Result<(String, &str)> {
    let s = s.trim_start();
    if !s.starts_with('"') {
        return Err(anyhow::anyhow!("Expected quoted string"));
    }
    let s = &s[1..];
    if let Some(idx) = s.find('"') {
        Ok((s[..idx].to_string(), &s[idx + 1..]))
    } else {
        Err(anyhow::anyhow!("Unterminated quoted string"))
    }
}

fn extract_quoted(s: &str) -> Result<String> {
    let (quoted, rest) = split_quoted(s)?;
    if !rest.trim().is_empty() {
        return Err(anyhow::anyhow!("Unexpected trailing content after quoted string: '{}'", rest));
    }
    Ok(quoted)
}

fn extract_string_array(s: &str) -> Result<Vec<String>> {
    let s = s.trim();
    if !s.starts_with('[') || !s.ends_with(']') {
        return Err(anyhow::anyhow!("Expected array to be enclosed in [...]"));
    }
    let inner = s[1..s.len() - 1].trim();
    if inner.is_empty() {
        return Ok(Vec::new());
    }
    
    let mut res = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    
    for c in inner.chars() {
        match c {
            '"' => {
                in_quotes = !in_quotes;
                current.push('"');
            }
            ',' if !in_quotes => {
                res.push(extract_quoted(current.trim())?);
                current.clear();
            }
            _ => {
                current.push(c);
            }
        }
    }
    let last = current.trim();
    if !last.is_empty() {
        res.push(extract_quoted(last)?);
    }
    
    Ok(res)
}

fn collect_block(lines: &[&str], start: usize) -> Result<(String, usize)> {
    let mut i = start;
    let mut block_lines = Vec::new();
    
    while i < lines.len() {
        let line = lines[i];
        if line.trim() == "}" {
            break;
        }
        block_lines.push(line);
        i += 1;
    }
    
    if i == lines.len() {
        return Err(anyhow::anyhow!("unterminated block starting around line {}", start));
    }
    
    let mut min_indent = None;
    for line in &block_lines {
        if !line.trim().is_empty() {
            let indent = line.chars().take_while(|c| c.is_whitespace()).count();
            min_indent = Some(min_indent.unwrap_or(indent).min(indent));
        }
    }
    
    let strip_chars = min_indent.unwrap_or(0);
    
    let mut body = Vec::new();
    for line in block_lines {
        if line.trim().is_empty() {
            body.push(String::new());
        } else {
            let stripped: String = line.chars().skip(strip_chars).collect();
            body.push(stripped);
        }
    }
    
    Ok((body.join("\n"), i))
}

pub fn parse_file(path: &std::path::Path) -> Result<Spec> {
    let content = std::fs::read_to_string(path)?;
    let lines: Vec<&str> = content.lines().collect();
    
    let mut name = None;
    let mut targets = Vec::new();
    let mut runner = None;
    let mut runner_args = Vec::new();
    let mut lang = None;
    let mut fixtures = Vec::new();
    let mut witnesses = Vec::new();
    
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();
        
        if trimmed.is_empty() || trimmed.starts_with('#') {
            i += 1;
            continue;
        }
        
        let (keyword, rest) = match trimmed.split_once(|c: char| c.is_whitespace()) {
            Some((k, r)) => (k, r.trim()),
            None => (trimmed, ""),
        };
        
        match keyword {
            "spec" => {
                if name.is_some() {
                    return Err(anyhow::anyhow!("Duplicate 'spec' directive at line {}", i + 1));
                }
                name = Some(extract_quoted(rest)?);
            }
            "target" => {
                targets.push(extract_quoted(rest)?);
            }
            "runner" => {
                if runner.is_some() {
                    return Err(anyhow::anyhow!("Duplicate 'runner' directive at line {}", i + 1));
                }
                runner = Some(rest.to_string());
            }
            "runner_args" => {
                runner_args = extract_string_array(rest)?;
            }
            "lang" => {
                lang = Some(rest.to_string());
            }
            "fixture" => {
                if !rest.ends_with('{') {
                    return Err(anyhow::anyhow!("Expected '{{' at the end of fixture at line {}", i + 1));
                }
                let f_name = rest.trim_end_matches('{').trim().to_string();
                let (body, next_i) = collect_block(&lines, i + 1)?;
                fixtures.push(Fixture { name: f_name, body });
                i = next_i;
            }
            "witness" => {
                let (w_name, remainder) = split_quoted(rest)?;
                let remainder = remainder.trim();
                if !remainder.ends_with('{') {
                    return Err(anyhow::anyhow!("Expected '{{' at the end of witness at line {}", i + 1));
                }
                let mode_str = remainder.trim_end_matches('{').trim();
                let kind = if mode_str == "hard" {
                    Kind::Hard
                } else if mode_str.starts_with("soft weight ") {
                    let weight_str = mode_str["soft weight ".len()..].trim();
                    let w: f64 = weight_str.parse().map_err(|e| anyhow::anyhow!("Failed to parse soft weight at line {}: {}", i + 1, e))?;
                    Kind::Soft(w)
                } else {
                    return Err(anyhow::anyhow!("Unknown witness mode '{}' at line {}", mode_str, i + 1));
                };
                
                let (body, next_i) = collect_block(&lines, i + 1)?;
                witnesses.push(Witness { name: w_name, kind, body });
                i = next_i;
            }
            _ => {
                return Err(anyhow::anyhow!("Unknown keyword at line {}: {}", i + 1, keyword));
            }
        }
        
        i += 1;
    }
    
    let name = name.ok_or_else(|| anyhow::anyhow!("Missing 'spec' directive"))?;
    if targets.is_empty() {
        return Err(anyhow::anyhow!("Missing 'target' directive"));
    }
    let runner = runner.ok_or_else(|| anyhow::anyhow!("Missing 'runner' directive"))?;
    let lang = lang.ok_or_else(|| anyhow::anyhow!("Missing 'lang' directive"))?;
    
    Ok(Spec {
        name,
        targets,
        runner,
        runner_args,
        lang,
        fixtures,
        witnesses,
    })
}
