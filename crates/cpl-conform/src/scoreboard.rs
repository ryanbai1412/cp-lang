use std::collections::BTreeMap;
use std::path::Path;

use crate::json::{self, Value};

pub const SCHEMA_VERSION: i64 = 2;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CaseOutcome {
    pub status: String,
    pub detail: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SuiteReport {
    pub total: usize,
    pub passed: usize,
    pub counts: BTreeMap<String, usize>,
    pub cases: BTreeMap<String, String>,
    pub details: BTreeMap<String, String>,
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
        SuiteReport {
            total,
            passed,
            counts,
            cases,
            details,
        }
    }

    pub fn to_json(&self) -> Value {
        let mut obj = BTreeMap::new();
        obj.insert("total".into(), Value::Int(self.total as i64));
        obj.insert("passed".into(), Value::Int(self.passed as i64));
        let counts: BTreeMap<String, Value> = self
            .counts
            .iter()
            .map(|(k, v)| (k.clone(), Value::Int(*v as i64)))
            .collect();
        obj.insert("counts".into(), Value::Object(counts));
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
        let total = obj
            .get("total")
            .and_then(Value::as_usize)
            .unwrap_or(cases.len());
        let passed = obj
            .get("passed")
            .and_then(Value::as_usize)
            .unwrap_or_else(|| counts.get("pass").copied().unwrap_or(0));
        Ok(SuiteReport {
            total,
            passed,
            counts,
            cases,
            details: BTreeMap::new(),
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
        let val = json::parse(text)?;
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
                None if !filtered => {
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
