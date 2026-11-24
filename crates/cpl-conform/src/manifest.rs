//! The I/O contract's cases as JSON (`cpl-conform conformance [filter] --manifest P`):
//! what each conformance case feeds a program and expects back, read by the
//! same header parser the suite runs on, so the browser (and any other side)
//! needs no parser of its own.
use std::collections::BTreeMap;
use std::path::Path;

use cpl_json::{Value, Writer};

use crate::conformance::{discover, header_input, header_values, parse_header, unquote, Expect};
use crate::scoreboard::Scoreboard;
use crate::{default_scoreboard_path, Suite};

/// How stdout is compared: the suite trims trailing newlines on both sides.
pub const TRIM_FINAL_NEWLINES: &str = "trim-final-newlines";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IoCase {
    /// Relative to `spec/conformance`, as the suite names it.
    pub path: String,
    pub source: String,
    pub input: String,
    /// The program runs (else the case only checks it, or runs another tool).
    pub run: bool,
    /// Expected stdout, compared by `compare`; `None` when the case doesn't fix it.
    pub stdout: Option<String>,
    pub compare: Option<String>,
    /// Expected exit status; `None` for a trap, which `trap` names instead.
    pub exit: Option<i32>,
    /// The trap code and the 1-based line it points at.
    pub trap: Option<(String, usize)>,
    /// Files beside the case (`// file:`), by path relative to it; `.cp` ones are modules.
    pub files: BTreeMap<String, String>,
    /// A page can run it: it runs with no flags beyond `run`, no judge and no C++ body.
    pub browser: bool,
    /// Why no side meets it yet: its `// gap:` line, else the suite's own record of it failing.
    pub gap: Option<String>,
}

impl IoCase {
    /// `failing` is what the suite last recorded for a case it doesn't pass.
    pub fn of(path: &str, text: &str, failing: Option<&str>) -> Result<IoCase, String> {
        let expect = parse_header(text)?;
        let tool = header_values(text, "tool:")
            .first()
            .map(|t| t.split_whitespace().collect::<Vec<_>>());
        let runs = match &tool {
            None => matches!(expect, Expect::OkOutput(_) | Expect::Trap { .. }),
            Some(words) => words.first() == Some(&"run"),
        };
        let plain = tool
            .as_ref()
            .is_none_or(|w| w[1..].iter().all(|a| *a == "{case}"));
        let mut files = BTreeMap::new();
        for spec in header_values(text, "file:") {
            let (rel, body) = spec
                .split_once(char::is_whitespace)
                .ok_or("`// file:` needs a path and a quoted body")?;
            files.insert(rel.to_string(), unquote(body.trim()));
        }
        let (stdout, exit, trap) = match expect {
            _ if !runs => (None, None, None),
            Expect::OkOutput(want) => (Some(want), Some(0), None),
            Expect::Ok => (None, Some(0), None),
            Expect::Exit(code) => (None, Some(code), None),
            Expect::Trap { code, line } => (None, None, Some((code, line))),
            Expect::Diag { .. } => (None, None, None),
        };
        let runs = runs && (exit.is_some() || trap.is_some());
        let browser = runs
            && plain
            && header_values(text, "judge:").is_empty()
            && !cpl_ir::has_cpp_body(text);
        let gap = (header_values(text, "gap:").first().copied())
            .or(failing)
            .map(str::to_string);
        Ok(IoCase {
            path: path.to_string(),
            source: text.to_string(),
            input: String::from_utf8_lossy(&header_input(text)).into_owned(),
            run: runs,
            compare: stdout.as_ref().map(|_| TRIM_FINAL_NEWLINES.to_string()),
            stdout,
            exit,
            trap,
            files,
            browser,
            gap,
        })
    }

    pub fn write(&self, w: &mut Writer) {
        w.object();
        w.key("path").str(&self.path);
        w.key("source").str(&self.source);
        w.key("input").str(&self.input);
        w.key("run").bool(self.run);
        w.key("stdout").opt_str(self.stdout.as_deref());
        w.key("compare").opt_str(self.compare.as_deref());
        w.key("exit");
        match self.exit {
            Some(code) => w.int(code),
            None => w.null(),
        };
        w.key("trap");
        match &self.trap {
            Some((code, line)) => w
                .object()
                .key("code")
                .str(code)
                .key("line")
                .int(*line)
                .end(),
            None => w.null(),
        };
        w.key("files").object();
        for (rel, body) in &self.files {
            w.key(rel).str(body);
        }
        w.end();
        w.key("browser").bool(self.browser);
        w.key("gap").opt_str(self.gap.as_deref());
        w.end();
    }

    pub fn read(v: &Value) -> Option<IoCase> {
        let text = |k: &str| v.get(k)?.as_str().map(str::to_string);
        let trap = match v.get("trap")? {
            Value::Null => None,
            t => Some((
                t.get("code")?.as_str()?.to_string(),
                t.get("line")?.as_usize()?,
            )),
        };
        Some(IoCase {
            path: text("path")?,
            source: text("source")?,
            input: text("input")?,
            run: v.get("run")?.as_bool()?,
            stdout: text("stdout"),
            compare: text("compare"),
            exit: v.get("exit")?.as_i64().map(|n| n as i32),
            trap,
            files: v
                .get("files")?
                .as_object()?
                .iter()
                .map(|(k, b)| Some((k.clone(), b.as_str()?.to_string())))
                .collect::<Option<_>>()?,
            browser: v.get("browser")?.as_bool()?,
            gap: text("gap"),
        })
    }
}

/// `{"cases": [...]}` for every conformance case `filter` selects, or the first case it can't read.
/// A case the scoreboard records as failing is a gap on every side.
pub fn manifest(root: &Path, filter: Option<&str>) -> Result<String, String> {
    let board = Scoreboard::load(&default_scoreboard_path(root))?;
    let recorded = board.suites.get(Suite::Conformance.name());
    let mut cases = Vec::new();
    for case in discover(root, filter) {
        let failing = recorded.and_then(|r| {
            let status = r.cases.get(&case.id).filter(|s| *s != "pass")?;
            Some(format!("the suite records it as `{status}`"))
        });
        let text = std::fs::read_to_string(&case.path).map_err(|e| format!("{}: {e}", case.id))?;
        let io = IoCase::of(&case.id, &text, failing.as_deref());
        cases.push(io.map_err(|e| format!("{}: {e}", case.id))?);
    }
    Ok(cpl_json::write_with(|w| {
        w.object().key("cases").array();
        for c in &cases {
            c.write(w);
        }
        w.end().end();
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn round_trip(id: &str) -> IoCase {
        let root = crate::find_repo_root().unwrap();
        let text = std::fs::read_to_string(root.join("spec/conformance").join(id)).unwrap();
        let case = IoCase::of(id, &text, None).unwrap();
        let json = cpl_json::write_with(|w| case.write(w));
        let back = IoCase::read(&cpl_json::parse(&json).unwrap()).unwrap();
        assert_eq!(back, case, "{id} reads back as it was written");
        case
    }

    #[test]
    fn an_interactive_case_round_trips() {
        let c = round_trip("core-06/interactor-send-recv.cp");
        assert!(c.run && c.browser);
        assert_eq!(c.input, "7 41\n");
        assert_eq!(c.stdout.as_deref(), Some("? 7\n! 42"));
        assert_eq!(c.compare.as_deref(), Some(TRIM_FINAL_NEWLINES));
        assert_eq!(c.exit, Some(0));
    }

    #[test]
    fn a_case_with_a_module_beside_it_round_trips() {
        let c = round_trip("core-11/library-module-uses-std.cp");
        assert!(c.run && c.browser);
        assert_eq!(c.files.len(), 2, "the package config and its module");
        assert_eq!(
            c.files["kit/m.cp"],
            "use std.math.{gcd}\npub fn g() -> Int = gcd(4, 6)\n"
        );
        assert_eq!(c.stdout.as_deref(), Some("2"));
    }

    #[test]
    fn a_check_only_case_round_trips_and_does_not_run() {
        let c = round_trip("core-06/check-accepts-valid-output.cp");
        assert!(!c.run && !c.browser);
        assert_eq!((c.stdout, c.exit, c.trap), (None, None, None));
        assert_eq!(c.files["in"], "5\n");
    }

    #[test]
    fn a_cpp_body_runs_but_not_on_a_page() {
        let c = round_trip("core-09/trusted-and-cpp.cp");
        assert!(c.run && !c.browser);
    }

    #[test]
    fn a_case_the_suite_fails_is_a_gap() {
        let text = "// expect: ok, output \"1\"\nprint(1)\n";
        let c = IoCase::of("x.cp", text, Some("the suite records it as `fail`")).unwrap();
        assert_eq!(c.gap.as_deref(), Some("the suite records it as `fail`"));
        let c = IoCase::of("x.cp", &format!("{text}// gap: not yet\n"), None).unwrap();
        assert_eq!(c.gap, None, "a `// gap:` line belongs in the header");
        let c = IoCase::of("x.cp", &text.replacen("\n", "\n// gap: not yet\n", 1), None).unwrap();
        assert_eq!(c.gap.as_deref(), Some("not yet"));
    }

    #[test]
    fn a_trap_names_its_code_and_line_not_an_exit() {
        let c = round_trip("core-11/library-module-no-prelude.cp");
        assert!(!c.run, "an E-code case only checks");
        let root = crate::find_repo_root().unwrap();
        let all = manifest(&root, Some("core-10/")).unwrap();
        let cases = cpl_json::parse(&all).unwrap();
        let cases: Vec<IoCase> = cases
            .get("cases")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .map(|v| IoCase::read(v).unwrap())
            .collect();
        assert!(cases
            .iter()
            .any(|c| c.trap.is_some() && c.exit.is_none() && c.run));
        assert!(cases.iter().all(|c| c.path.starts_with("core-10/")));
    }
}
