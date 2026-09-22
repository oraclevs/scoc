use serde_json::{Map, Value};

use super::dynamic_table::{dynamic_table, HeaderPolicy};
use super::normalize::{normalized_key, parse_human_bytes, percent, scalar};
use crate::ScocError;

fn table(text: &str) -> Result<Vec<Map<String, Value>>, ScocError> {
    let lines: Vec<&str> = text.lines().collect();
    dynamic_table(&lines, HeaderPolicy::FirstNonEmpty)
}

fn pair_bytes(value: &str) -> (Option<u64>, Option<u64>) {
    let mut parts = value.split('/').map(str::trim);
    (
        parts.next().and_then(parse_human_bytes),
        parts.next().and_then(parse_human_bytes),
    )
}

fn value_string(value: &Value) -> Option<&str> {
    value.as_str()
}

fn docker_stats(text: &str) -> Result<Value, ScocError> {
    let mut rows = table(text)?;
    for row in &mut rows {
        if let Some(raw) = row.get("cpu").and_then(value_string) {
            if let Some(v) = percent(raw) {
                row.insert("cpu_percent".into(), Value::from(v));
            }
        }
        if let Some(raw) = row.get("mem").and_then(value_string) {
            if let Some(v) = percent(raw) {
                row.insert("memory_percent".into(), Value::from(v));
            }
        }
        if let Some(raw) = row
            .get("mem_usage_limit")
            .and_then(value_string)
            .map(str::to_string)
        {
            row.insert("memory_usage".into(), Value::String(raw.clone()));
            let (used, limit) = pair_bytes(&raw);
            if let Some(v) = used {
                row.insert("memory_used_bytes".into(), Value::from(v));
            }
            if let Some(v) = limit {
                row.insert("memory_limit_bytes".into(), Value::from(v));
            }
        }
        if let Some(raw) = row
            .get("net_i_o")
            .and_then(value_string)
            .map(str::to_string)
        {
            let (input, output) = pair_bytes(&raw);
            if let Some(v) = input {
                row.insert("network_input_bytes".into(), Value::from(v));
            }
            if let Some(v) = output {
                row.insert("network_output_bytes".into(), Value::from(v));
            }
        }
        if let Some(raw) = row
            .get("block_i_o")
            .and_then(value_string)
            .map(str::to_string)
        {
            let (input, output) = pair_bytes(&raw);
            if let Some(v) = input {
                row.insert("block_input_bytes".into(), Value::from(v));
            }
            if let Some(v) = output {
                row.insert("block_output_bytes".into(), Value::from(v));
            }
        }
    }
    Ok(Value::Array(rows.into_iter().map(Value::Object).collect()))
}

fn pip_freeze(text: &str) -> Value {
    let mut out = Vec::new();
    for line in text.lines().map(str::trim).filter(|line| !line.is_empty()) {
        let mut row = Map::new();
        if !line.starts_with("-e ") {
            if let Some((name, version)) = line.split_once("==") {
                row.insert("name".into(), Value::String(name.to_string()));
                row.insert("version".into(), Value::String(version.to_string()));
                row.insert("requirement".into(), Value::String(line.to_string()));
                out.push(Value::Object(row));
                continue;
            }
        }
        row.insert("requirement".into(), Value::String(line.to_string()));
        if let Some((name, reference)) = line.split_once(" @ ") {
            row.insert("name".into(), Value::String(name.trim().to_string()));
            row.insert(
                "reference".into(),
                Value::String(reference.trim().to_string()),
            );
        }
        out.push(Value::Object(row));
    }
    Value::Array(out)
}

fn unquote(value: &str) -> &str {
    let trimmed = value.trim();
    if trimmed.len() >= 2 {
        let bytes = trimmed.as_bytes();
        if (bytes[0] == b'\'' && bytes[trimmed.len() - 1] == b'\'')
            || (bytes[0] == b'"' && bytes[trimmed.len() - 1] == b'"')
        {
            return &trimmed[1..trimmed.len() - 1];
        }
    }
    trimmed
}

fn go_env(text: &str) -> Value {
    let mut out = Map::new();
    for line in text.lines().map(str::trim).filter(|line| !line.is_empty()) {
        if let Some((key, value)) = line.split_once('=') {
            out.insert(normalized_key(key), scalar(unquote(value)));
        }
    }
    Value::Object(out)
}

fn terraform_workspace(text: &str) -> Value {
    let rows = text
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                return None;
            }
            let current = trimmed.starts_with('*');
            let name = trimmed.trim_start_matches('*').trim();
            let mut row = Map::new();
            row.insert("name".into(), Value::String(name.to_string()));
            row.insert("current".into(), Value::Bool(current));
            Some(Value::Object(row))
        })
        .collect();
    Value::Array(rows)
}

fn address_lines(text: &str) -> Value {
    Value::Array(
        text.lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(|line| {
                let mut row = Map::new();
                row.insert("address".into(), Value::String(line.to_string()));
                Value::Object(row)
            })
            .collect(),
    )
}

fn git_status(text: &str) -> Value {
    let mut rows = Vec::new();
    let porcelain = text
        .lines()
        .all(|line| line.trim().is_empty() || line.starts_with("##") || line.len() >= 3);
    if porcelain {
        for line in text.lines().filter(|line| !line.trim().is_empty()) {
            if let Some(branch) = line.strip_prefix("## ") {
                let mut row = Map::new();
                row.insert("kind".into(), Value::String("branch".into()));
                row.insert("value".into(), Value::String(branch.to_string()));
                rows.push(Value::Object(row));
                continue;
            }
            if line.len() < 3 {
                continue;
            }
            let (status, path_part) = line.split_at(2);
            let mut row = Map::new();
            row.insert(
                "index_status".into(),
                Value::String(status.chars().next().unwrap_or(' ').to_string()),
            );
            row.insert(
                "worktree_status".into(),
                Value::String(status.chars().nth(1).unwrap_or(' ').to_string()),
            );
            let path_part = path_part.trim();
            if let Some((old, new)) = path_part.split_once(" -> ") {
                row.insert("original_path".into(), Value::String(old.to_string()));
                row.insert("path".into(), Value::String(new.to_string()));
            } else {
                row.insert("path".into(), Value::String(path_part.to_string()));
            }
            rows.push(Value::Object(row));
        }
        return Value::Array(rows);
    }
    Value::Array(
        text.lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| {
                let mut row = Map::new();
                row.insert("value".into(), Value::String(line.trim().to_string()));
                Value::Object(row)
            })
            .collect(),
    )
}

fn git_branch(text: &str) -> Value {
    Value::Array(
        text.lines()
            .filter_map(|line| {
                let trimmed = line.trim_end();
                if trimmed.is_empty() {
                    return None;
                }
                let current = trimmed.starts_with('*');
                let body = trimmed.trim_start().trim_start_matches('*').trim();
                let mut parts = body.split_whitespace();
                let name = parts.next().unwrap_or("");
                let rest = parts.collect::<Vec<_>>().join(" ");
                let mut row = Map::new();
                row.insert("name".into(), Value::String(name.to_string()));
                row.insert("current".into(), Value::Bool(current));
                if !rest.is_empty() {
                    row.insert("details".into(), Value::String(rest));
                }
                Some(Value::Object(row))
            })
            .collect(),
    )
}

fn git_remote(text: &str) -> Value {
    Value::Array(
        text.lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.is_empty() {
                    return None;
                }
                let mut row = Map::new();
                row.insert("name".into(), Value::String(parts[0].to_string()));
                if let Some(url) = parts.get(1) {
                    row.insert("url".into(), Value::String((*url).to_string()));
                }
                if let Some(kind) = parts.get(2) {
                    row.insert(
                        "kind".into(),
                        Value::String(kind.trim_matches(|c| c == '(' || c == ')').to_string()),
                    );
                }
                Some(Value::Object(row))
            })
            .collect(),
    )
}

fn git_submodule(text: &str) -> Value {
    Value::Array(
        text.lines()
            .filter_map(|line| {
                if line.trim().is_empty() {
                    return None;
                }
                let marker = line.chars().next().unwrap_or(' ');
                let rest = line.get(1..).unwrap_or(line).trim();
                let mut parts = rest.splitn(3, char::is_whitespace);
                let commit = parts.next().unwrap_or("");
                let path = parts.next().unwrap_or("");
                let details = parts.next().unwrap_or("").trim();
                let mut row = Map::new();
                row.insert("status".into(), Value::String(marker.to_string()));
                row.insert("commit".into(), Value::String(commit.to_string()));
                row.insert("path".into(), Value::String(path.to_string()));
                if !details.is_empty() {
                    row.insert("details".into(), Value::String(details.to_string()));
                }
                Some(Value::Object(row))
            })
            .collect(),
    )
}

fn npm_run(text: &str) -> Value {
    let mut rows: Vec<Value> = Vec::new();
    let mut current: Option<Map<String, Value>> = None;
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let indent = line.chars().take_while(|c| c.is_whitespace()).count();
        let trimmed = line.trim();
        if indent <= 2
            && !trimmed.ends_with(':')
            && !trimmed.starts_with('>')
            && !trimmed.contains(" available via `npm run-script`")
        {
            if let Some(row) = current.take() {
                rows.push(Value::Object(row));
            }
            let mut row = Map::new();
            row.insert("script".into(), Value::String(trimmed.to_string()));
            current = Some(row);
        } else if indent >= 4 {
            if let Some(row) = current.as_mut() {
                row.insert("command".into(), Value::String(trimmed.to_string()));
            }
        }
    }
    if let Some(row) = current {
        rows.push(Value::Object(row));
    }
    Value::Array(rows)
}

fn pip_check(text: &str) -> Value {
    let trimmed = text.trim();
    if trimmed.eq_ignore_ascii_case("No broken requirements found.") {
        let mut row = Map::new();
        row.insert("ok".into(), Value::Bool(true));
        row.insert("message".into(), Value::String(trimmed.to_string()));
        return Value::Array(vec![Value::Object(row)]);
    }
    Value::Array(
        text.lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| {
                let mut row = Map::new();
                row.insert("ok".into(), Value::Bool(false));
                row.insert("message".into(), Value::String(line.trim().to_string()));
                Value::Object(row)
            })
            .collect(),
    )
}

fn marker_list(text: &str) -> Value {
    Value::Array(
        text.lines()
            .filter_map(|line| {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    return None;
                }
                let default = trimmed.ends_with("(default)");
                let installed = trimmed.ends_with("(installed)") || default;
                let name = trimmed
                    .trim_end_matches("(default)")
                    .trim_end_matches("(installed)")
                    .trim();
                let mut row = Map::new();
                row.insert("name".into(), Value::String(name.to_string()));
                row.insert("default".into(), Value::Bool(default));
                row.insert("installed".into(), Value::Bool(installed));
                Some(Value::Object(row))
            })
            .collect(),
    )
}

fn flutter_doctor(text: &str) -> Value {
    let mut out = Vec::new();
    let mut current: Option<Map<String, Value>> = None;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let status = if trimmed.starts_with("[✓]") {
            Some("ok")
        } else if trimmed.starts_with("[!]") {
            Some("warning")
        } else if trimmed.starts_with("[✗]") || trimmed.starts_with("[x]") {
            Some("error")
        } else if trimmed.starts_with("[-]") {
            Some("info")
        } else {
            None
        };
        if let Some(status) = status {
            if let Some(row) = current.take() {
                out.push(Value::Object(row));
            }
            let title = trimmed
                .char_indices()
                .nth(3)
                .map(|(idx, _)| trimmed[idx..].trim())
                .unwrap_or(trimmed);
            let mut row = Map::new();
            row.insert("status".into(), Value::String(status.into()));
            row.insert("title".into(), Value::String(title.to_string()));
            row.insert("details".into(), Value::Array(Vec::new()));
            current = Some(row);
        } else if let Some(row) = current.as_mut() {
            if let Some(Value::Array(details)) = row.get_mut("details") {
                details.push(Value::String(trimmed.to_string()));
            }
        }
    }
    if let Some(row) = current {
        out.push(Value::Object(row));
    }
    Value::Array(out)
}

pub(crate) fn parse_native(name: &str, text: &str) -> Option<Result<Value, ScocError>> {
    let result = match name {
        "docker-stats" => return Some(docker_stats(text)),
        "pip-freeze" => pip_freeze(text),
        "pip-check" => pip_check(text),
        "go-env" => go_env(text),
        "terraform-state-list" => address_lines(text),
        "terraform-workspace-list" => terraform_workspace(text),
        "git-status" => git_status(text),
        "git-branch" => git_branch(text),
        "git-remote" => git_remote(text),
        "git-submodule" => git_submodule(text),
        "npm-run" => npm_run(text),
        "rustup-toolchains" | "rustup-targets" => marker_list(text),
        "flutter-doctor" => flutter_doctor(text),
        _ => return None,
    };
    Some(Ok(result))
}
