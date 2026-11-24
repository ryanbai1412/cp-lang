use std::collections::BTreeMap;
use std::path::Path;

use cpl_json::{self as json, Value};

pub const SCHEMA_VERSION: i64 = 2;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CaseOutcome {
    pub status: String,
    pub detail: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CauseEntry {
    pub cause: String,
    pub count: usize,
    pub examples: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SuiteReport {
    pub total: usize,
    pub passed: usize,
    pub strict_passed: Option<usize>,
    pub counts: BTreeMap<String, usize>,
    pub cases: BTreeMap<String, String>,
    pub details: BTreeMap<String, String>,
    pub causes: Vec<CauseEntry>,
}

/// Extracts the first `error[CODE]: msg` / `warning[CODE]: msg` / `lint[CODE]: msg`
/// from `s`, returning `(code, first_line_message)`.
fn extract_first_diag(s: &str) -> Option<(String, String)> {
    for prefix in ["error[", "warning[", "lint["] {
        if let Some(pos) = s.find(prefix) {
            let rest = &s[pos + prefix.len()..];
            if let Some(end) = rest.find(']') {
                let code = rest[..end].trim().to_string();
                let after = rest[end + 1..].trim_start_matches(':').trim_start();
                let line = after.lines().next().unwrap_or("").trim();
                return Some((code, normalize_message(line)));
            }
        }
    }
    None
}

/// Replaces backtick-quoted identifiers/types, quoted literals, and absolute paths
/// so equivalent diagnostics group under one root cause.
pub fn normalize_message(msg: &str) -> String {
    let mut out = String::with_capacity(msg.len());
    let bytes = msg.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'`' => {
                if let Some(end) = msg[i + 1..].find('`') {
                    let inner = &msg[i + 1..i + 1 + end];
                    if inner == "todo" {
                        out.push_str("`todo`");
                    } else {
                        out.push_str("`<id>`");
                    }
                    i += end + 2;
                    continue;
                }
                out.push('`');
                i += 1;
            }
            b'"' => {
                let mut j = i + 1;
                while j < bytes.len() {
                    if bytes[j] == b'\\' && j + 1 < bytes.len() {
                        j += 2;
                    } else if bytes[j] == b'"' {
                        j += 1;
                        break;
                    } else {
                        j += 1;
                    }
                }
                out.push_str("\"<str>\"");
                i = j;
            }
            _ => {
                let ch = msg[i..].chars().next().unwrap();
                out.push(ch);
                i += ch.len_utf8();
            }
        }
    }
    let trimmed = out.trim();
    if trimmed.len() > 140 {
        let mut end = 140;
        while !trimmed.is_char_boundary(end) {
            end -= 1;
        }
        format!("{}...", &trimmed[..end])
    } else {
        trimmed.to_string()
    }
}

/// Classifies a non-passing case outcome into a normalized root cause string.
/// Returns `None` for non-failure statuses (`pass`, `skip`, `untested`).
pub fn classify_cause(status: &str, detail: &str) -> Option<String> {
    if matches!(status, "pass" | "skip" | "untested") {
        return None;
    }
    if status == "panic" || detail.starts_with("panic:") {
        let line = detail.lines().next().unwrap_or("panic").trim();
        return Some(format!("panic: {}", normalize_message(line)));
    }
    if status == "wrong" {
        return Some("wrong output".into());
    }
    if status == "timeout" {
        return Some("timeout".into());
    }
    if status == "cxx_err" {
        let mut msg = "g++ error";
        for line in detail.lines() {
            let t = line.trim();
            if let Some(pos) = t.find("error:") {
                msg = t[pos..].trim();
                break;
            }
            if let Some(pos) = t.find(": In ") {
                msg = t[pos + 2..].trim();
            }
        }
        return Some(format!("cxx_err: {}", normalize_message(msg)));
    }
    if let Some(rest) = detail.strip_prefix("wanted ") {
        if let Some((want_part, got_part)) = rest.split_once("; got") {
            let want = want_part.trim();
            let got = got_part.trim_start_matches(':').trim();
            if want == "ok" || want.starts_with("output ") {
                if let Some((code, msg)) = extract_first_diag(got) {
                    return Some(format!("{code}: {msg}"));
                }
                if want.starts_with("output ") {
                    return Some("wrong output".into());
                }
                return Some(format!(
                    "wanted {want}; got {}",
                    normalize_message(got.lines().next().unwrap_or(""))
                ));
            }
            let is_trap = want.starts_with("trap ") || want.starts_with("a trap ");
            let expected_code = want
                .strip_prefix("a trap ")
                .or_else(|| want.strip_prefix("trap "))
                .unwrap_or(want)
                .split_whitespace()
                .next()
                .unwrap_or(want);
            if got.contains(": no errors") || got.is_empty() {
                return Some(format!("missing {expected_code} (no error emitted)"));
            }
            if let Some((actual_code, msg)) = extract_first_diag(got) {
                if actual_code == expected_code {
                    return Some(format!("wrong line for {expected_code}: {msg}"));
                }
                return Some(format!("wanted {expected_code}; got {actual_code}: {msg}"));
            }
            if is_trap && got.starts_with("exit 0") {
                return Some(format!("missing trap {expected_code} (exited 0)"));
            }
            return Some(format!(
                "wanted {expected_code}; got {}",
                normalize_message(got.lines().next().unwrap_or(""))
            ));
        }
    }
    if let Some((code, msg)) = extract_first_diag(detail) {
        return Some(format!("{code}: {msg}"));
    }
    let first = detail.lines().next().unwrap_or(status).trim();
    if first.is_empty() {
        Some(status.to_string())
    } else {
        Some(format!("{status}: {}", normalize_message(first)))
    }
}

pub fn compute_causes(
    cases: &BTreeMap<String, String>,
    details: &BTreeMap<String, String>,
) -> Vec<CauseEntry> {
    let mut grouped: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (id, status) in cases {
        let detail = details.get(id).map(String::as_str).unwrap_or("");
        if let Some(cause) = classify_cause(status, detail) {
            grouped.entry(cause).or_default().push(id.clone());
        }
    }
    let mut entries: Vec<CauseEntry> = grouped
        .into_iter()
        .map(|(cause, mut ids)| {
            ids.sort();
            let count = ids.len();
            ids.truncate(5);
            CauseEntry {
                cause,
                count,
                examples: ids,
            }
        })
        .collect();
    entries.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.cause.cmp(&b.cause)));
    entries
}

impl SuiteReport {
    pub fn from_outcomes(outcomes: BTreeMap<String, CaseOutcome>) -> Self {
        let mut counts: BTreeMap<String, usize> = BTreeMap::new();
        let mut cases = BTreeMap::new();
        let mut details = BTreeMap::new();
        for (id, out) in outcomes {
            *counts.entry(out.status.clone()).or_insert(0) += 1;
            if !out.detail.is_empty() {
                details.insert(id.clone(), out.detail);
            }
            cases.insert(id, out.status);
        }
        let passed = counts.get("pass").copied().unwrap_or(0);
        let total = cases.len();
        let causes = compute_causes(&cases, &details);
        SuiteReport {
            total,
            passed,
            strict_passed: None,
            counts,
            cases,
            details,
            causes,
        }
    }

    pub fn to_json(&self) -> Value {
        let mut obj = BTreeMap::new();
        obj.insert("total".into(), Value::Int(self.total as i64));
        obj.insert("passed".into(), Value::Int(self.passed as i64));
        if let Some(strict) = self.strict_passed {
            obj.insert("strict_passed".into(), Value::Int(strict as i64));
        }
        let counts: BTreeMap<String, Value> = self
            .counts
            .iter()
            .map(|(k, v)| (k.clone(), Value::Int(*v as i64)))
            .collect();
        obj.insert("counts".into(), Value::Object(counts));
        if !self.causes.is_empty() {
            let causes: Vec<Value> = self
                .causes
                .iter()
                .map(|c| {
                    let mut entry = BTreeMap::new();
                    entry.insert("cause".into(), Value::String(c.cause.clone()));
                    entry.insert("count".into(), Value::Int(c.count as i64));
                    entry.insert(
                        "examples".into(),
                        Value::Array(c.examples.iter().cloned().map(Value::String).collect()),
                    );
                    Value::Object(entry)
                })
                .collect();
            obj.insert("causes".into(), Value::Array(causes));
        }
        let cases: BTreeMap<String, Value> = self
            .cases
            .iter()
            .map(|(k, v)| (k.clone(), Value::String(v.clone())))
            .collect();
        obj.insert("cases".into(), Value::Object(cases));
        Value::Object(obj)
    }

    pub fn from_json(val: &Value) -> Result<Self, String> {
        let obj = val.as_object().ok_or("suite must be a JSON object")?;
        let mut cases = BTreeMap::new();
        if let Some(c) = obj.get("cases").and_then(Value::as_object) {
            for (k, v) in c {
                let s = v
                    .as_str()
                    .ok_or_else(|| format!("case `{k}` status must be a string"))?;
                cases.insert(k.clone(), s.to_string());
            }
        }
        let mut counts = BTreeMap::new();
        if let Some(c) = obj.get("counts").and_then(Value::as_object) {
            for (k, v) in c {
                let n = v
                    .as_usize()
                    .ok_or_else(|| format!("count `{k}` must be an integer"))?;
                counts.insert(k.clone(), n);
            }
        } else {
            for status in cases.values() {
                *counts.entry(status.clone()).or_insert(0) += 1;
            }
        }
        let mut causes = Vec::new();
        if let Some(arr) = obj.get("causes").and_then(Value::as_array) {
            for item in arr {
                if let Some(o) = item.as_object() {
                    let cause = o
                        .get("cause")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string();
                    let count = o.get("count").and_then(Value::as_usize).unwrap_or(0);
                    let examples = o
                        .get("examples")
                        .and_then(Value::as_array)
                        .map(|a| {
                            a.iter()
                                .filter_map(Value::as_str)
                                .map(str::to_string)
                                .collect()
                        })
                        .unwrap_or_default();
                    if !cause.is_empty() {
                        causes.push(CauseEntry {
                            cause,
                            count,
                            examples,
                        });
                    }
                }
            }
        }
        let total = obj
            .get("total")
            .and_then(Value::as_usize)
            .unwrap_or(cases.len());
        let passed = obj
            .get("passed")
            .and_then(Value::as_usize)
            .unwrap_or_else(|| counts.get("pass").copied().unwrap_or(0));
        let strict_passed = obj.get("strict_passed").and_then(Value::as_usize);
        Ok(SuiteReport {
            total,
            passed,
            strict_passed,
            counts,
            cases,
            details: BTreeMap::new(),
            causes,
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Scoreboard {
    pub suites: BTreeMap<String, SuiteReport>,
}

impl Scoreboard {
    pub fn to_json_string(&self) -> String {
        let mut root = BTreeMap::new();
        root.insert("schema_version".into(), Value::Int(SCHEMA_VERSION));
        let suites: BTreeMap<String, Value> = self
            .suites
            .iter()
            .map(|(name, report)| (name.clone(), report.to_json()))
            .collect();
        root.insert("suites".into(), Value::Object(suites));
        json::to_string_pretty(&Value::Object(root))
    }

    pub fn from_json_str(text: &str) -> Result<Self, String> {
        let val = json::parse(text).map_err(|e| e.to_string())?;
        let suites_obj = val
            .get("suites")
            .and_then(Value::as_object)
            .ok_or("scoreboard JSON missing `suites` object")?;
        let mut suites = BTreeMap::new();
        for (name, sval) in suites_obj {
            suites.insert(name.clone(), SuiteReport::from_json(sval)?);
        }
        Ok(Scoreboard { suites })
    }

    pub fn load(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| format!("cannot read `{}`: {e}", path.display()))?;
        Self::from_json_str(&text)
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("cannot create `{}`: {e}", parent.display()))?;
            }
        }
        std::fs::write(path, self.to_json_string())
            .map_err(|e| format!("cannot write `{}`: {e}", path.display()))
    }

    /// Merges updated suites into `self`, preserving suites that were not run.
    pub fn merge_from(&mut self, updated: &Scoreboard) {
        for (name, report) in &updated.suites {
            self.suites.insert(name.clone(), report.clone());
        }
    }
}

/// Cases that passed in `prev` (main's last green run, kept by CI) and no
/// longer pass in `current`, skipping any `baseline` already records: a
/// recorded pass is `check_ratchet`'s to report, a recorded failure is a known
/// gap. This covers cases added since the committed scoreboard was written.
/// A case gone from `current` was deleted or renamed, so it is not reported.
pub fn check_prev(baseline: &Scoreboard, prev: &Scoreboard, current: &Scoreboard) -> Vec<String> {
    let mut regressions = Vec::new();
    for (suite_name, cur) in &current.suites {
        let Some(prev) = prev.suites.get(suite_name) else {
            continue;
        };
        let recorded = baseline.suites.get(suite_name).map(|b| &b.cases);
        for (case_id, prev_status) in &prev.cases {
            if prev_status != "pass" || recorded.is_some_and(|r| r.contains_key(case_id)) {
                continue;
            }
            match cur.cases.get(case_id) {
                Some(status) if status != "pass" => {
                    let detail = cur
                        .details
                        .get(case_id)
                        .map(|d| format!(" ({d})"))
                        .unwrap_or_default();
                    regressions.push(format!(
                        "{suite_name}/{case_id}: regressed from pass on main's last green run to {status}{detail}"
                    ));
                }
                _ => {}
            }
        }
    }
    regressions
}

fn case_file(case_id: &str) -> &str {
    case_id.split('#').next().unwrap_or(case_id)
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RatchetReport {
    pub regressions: Vec<String>,
    pub improvements: Vec<String>,
}

/// Checks `current` against `baseline` for every suite present in `current`.
/// When `filtered` is true, total pass-count drops are not checked (only per-case
/// regressions among cases present in `current`).
pub fn check_ratchet(baseline: &Scoreboard, current: &Scoreboard, filtered: bool) -> RatchetReport {
    let mut report = RatchetReport::default();
    for (suite_name, cur) in &current.suites {
        let Some(base) = baseline.suites.get(suite_name) else {
            continue;
        };
        if !filtered && cur.passed < base.passed {
            report.regressions.push(format!(
                "{suite_name}: pass count dropped from {} to {} (-{})",
                base.passed,
                cur.passed,
                base.passed - cur.passed
            ));
        }
        // A case keyed `file#hash` (a doc example) changes key when its text is
        // edited, so a vanished one only regresses if its file passes fewer cases.
        let file_passes = |cases: &BTreeMap<String, String>, file: &str| {
            cases
                .iter()
                .filter(|(id, st)| *st == "pass" && case_file(id) == file)
                .count()
        };
        for (case_id, base_status) in &base.cases {
            if base_status != "pass" {
                continue;
            }
            match cur.cases.get(case_id) {
                Some(cur_status) if cur_status == "pass" => {}
                Some(cur_status) => {
                    let detail = cur
                        .details
                        .get(case_id)
                        .map(|d| format!(" ({d})"))
                        .unwrap_or_default();
                    report.regressions.push(format!(
                        "{suite_name}/{case_id}: regressed from pass to {cur_status}{detail}"
                    ));
                }
                None if !filtered
                    && (!case_id.contains('#')
                        || file_passes(&cur.cases, case_file(case_id))
                            < file_passes(&base.cases, case_file(case_id))) =>
                {
                    report.regressions.push(format!(
                        "{suite_name}/{case_id}: previously passing case disappeared"
                    ));
                }
                None => {}
            }
        }
        for (case_id, cur_status) in &cur.cases {
            if cur_status == "pass" && base.cases.get(case_id).map(String::as_str) != Some("pass") {
                report
                    .improvements
                    .push(format!("{suite_name}/{case_id}: newly passing"));
            }
        }
    }
    report
}
