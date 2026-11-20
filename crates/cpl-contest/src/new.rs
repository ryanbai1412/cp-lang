//! `cpl new`: a problem from its constraints.
//!
//! ```text
//! cpl new d --constraints "n, q: 1..=2e5; a: [n]0..=1e9; queries: [q](1..=n, 1..=n)"
//! ```
//!
//! writes `d.cp` with the input already read into typed values, which is
//! also everything `cpl stress` and `cpl bench` need to make inputs:
//!
//! ```cp
//! let (n, q): (1..=2e5, 1..=2e5) = read()
//! let a: [0..n]0..=1e9 = read()
//! let queries: [0..q](1..=n, 1..=n) = read()
//! ```

use crate::args::Args;
use crate::json::{self, Json};
use crate::problem::{tests_dir, Limits};
use cpl_stress::InputSpec;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

pub const USAGE: &str = "\
usage: cpl new <name> [options]

Creates <name>.cp and <name>.tests/.

options:
  --constraints TEXT   the input, as `name: type` separated by `;`:
                       \"n, q: 1..=2e5; a: [n]0..=1e9\"
  --companion FILE     a Competitive Companion problem (JSON; `-` reads
                       stdin): its title, limits and samples
  --time-limit MS      the problem's time limit in milliseconds
  --memory-limit MB    the problem's memory limit in megabytes
";

/// What is known about a problem before any code is written.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Scaffold {
    pub title: Option<String>,
    pub url: Option<String>,
    pub limits: Option<Limits>,
    pub interactive: bool,
    /// The `let ... = read()` lines.
    pub reads: Vec<String>,
    /// Sample inputs with their outputs.
    pub samples: Vec<(String, String)>,
}

/// `[n]T` is `n` values: `[0..n]T`. A range or a domain is left alone.
fn expand_lengths(ty: &str) -> String {
    let mut out = String::new();
    let mut rest = ty;
    while let Some(open) = rest.find('[') {
        let Some(close) = rest[open..].find(']').map(|at| open + at) else {
            break;
        };
        let inner = rest[open + 1..close].trim();
        let is_domain = inner.chars().next().is_some_and(char::is_uppercase)
            && inner.chars().all(|c| c.is_alphanumeric() || c == '_');
        out.push_str(&rest[..=open]);
        if inner.contains("..") || is_domain || inner.is_empty() {
            out.push_str(inner);
        } else {
            out.push_str(&format!("0..{inner}"));
        }
        out.push(']');
        rest = &rest[close + 1..];
    }
    out + rest
}

/// Splits on `sep` outside brackets.
fn split_top(text: &str, sep: &[char]) -> Vec<String> {
    let mut parts = vec![String::new()];
    let mut depth = 0i32;
    for c in text.chars() {
        match c {
            '(' | '[' => depth += 1,
            ')' | ']' => depth -= 1,
            _ => {}
        }
        if depth == 0 && sep.contains(&c) {
            parts.push(String::new());
        } else {
            parts.last_mut().unwrap().push(c);
        }
    }
    parts
}

/// Turns `n, q: 1..=2e5; a: [n]0..=1e9` into `let ... = read()` lines.
pub fn reads(constraints: &str) -> Result<Vec<String>, String> {
    let mut lines = Vec::new();
    for entry in split_top(constraints, &[';', '\n']) {
        let entry = entry.trim();
        if entry.is_empty() {
            continue;
        }
        let Some((names, ty)) = entry.split_once(':') else {
            return Err(format!("`{entry}` has no type; write `n: 1..=2e5`"));
        };
        let names: Vec<&str> = names.split(',').map(str::trim).collect();
        let is_name = |name: &&str| {
            name.chars()
                .next()
                .is_some_and(|c| c.is_ascii_lowercase() || c == '_')
                && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        };
        if let Some(bad) = names.iter().find(|name| !is_name(name)) {
            return Err(format!(
                "`{bad}` is not a name; names look like `n` or `max_w`"
            ));
        }
        let ty = expand_lengths(ty.trim());
        if ty.is_empty() {
            return Err(format!("`{entry}` has no type; write `n: 1..=2e5`"));
        }
        lines.push(match names.as_slice() {
            [name] => format!("let {name}: {ty} = read()"),
            names => format!(
                "let ({}): ({}) = read()",
                names.join(", "),
                vec![ty.as_str(); names.len()].join(", ")
            ),
        });
    }
    // The types must be ones the generator can make inputs from, or the
    // scaffold would not stress.
    let source = lines.join("\n") + "\n";
    if let Err(error) = InputSpec::from_source(&source) {
        let line = source[..error.span.start as usize].matches('\n').count();
        return Err(format!("in `{}`: {}", lines[line], error.message));
    }
    Ok(lines)
}

impl Scaffold {
    /// Reads a Competitive Companion problem.
    pub fn from_companion(text: &str) -> Result<Scaffold, String> {
        let problem = json::parse(text).map_err(|e| format!("this is not JSON: {e}"))?;
        if !matches!(problem, Json::Object(_)) {
            return Err("a Competitive Companion problem is a JSON object".to_string());
        }
        let text_of = |key: &str| problem.get(key).and_then(Json::as_str).map(str::to_string);
        let defaults = Limits::default();
        let limits = Limits {
            time: problem
                .get("timeLimit")
                .and_then(Json::as_u64)
                .map_or(defaults.time, Duration::from_millis),
            memory_kb: problem
                .get("memoryLimit")
                .and_then(Json::as_u64)
                .map_or(defaults.memory_kb, |mb| mb * 1024),
        };
        let samples = problem
            .get("tests")
            .map(Json::as_array)
            .unwrap_or_default()
            .iter()
            .filter_map(|test| {
                let input = test.get("input")?.as_str()?;
                let output = test.get("output").and_then(Json::as_str).unwrap_or("");
                Some((input.to_string(), output.to_string()))
            })
            .collect();
        Ok(Scaffold {
            title: text_of("name"),
            url: text_of("url"),
            limits: Some(limits),
            interactive: problem.get("interactive") == Some(&Json::Bool(true)),
            reads: Vec::new(),
            samples,
        })
    }

    /// A file name for the problem from its title: `D. Range Max` is `d`.
    pub fn short_name(&self) -> Option<String> {
        let title = self.title.as_deref()?;
        let first: String = title
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric())
            .collect::<String>()
            .to_ascii_lowercase();
        (!first.is_empty()).then_some(first)
    }

    /// The text of the new source file. `template` is the project's
    /// `template.cp`, put after the reads.
    pub fn source(&self, template: Option<&str>) -> String {
        let mut out = String::new();
        for line in [&self.title, &self.url].into_iter().flatten() {
            out.push_str(&format!("// {line}\n"));
        }
        if let Some(limits) = self.limits {
            out.push_str(&limits.header());
            out.push('\n');
        }
        if self.interactive {
            out.push_str("// interactive: test with `cpl test <name> --interactor <program>`\n");
        }
        if !out.is_empty() {
            out.push('\n');
        }
        for line in &self.reads {
            out.push_str(line);
            out.push('\n');
        }
        match template.map(str::trim_end) {
            Some(template) if !template.is_empty() => {
                if !self.reads.is_empty() {
                    out.push('\n');
                }
                out.push_str(template);
                out.push('\n');
            }
            _ if self.reads.is_empty() => out.push_str("let n: 1..=2e5 = read()\n"),
            _ => {}
        }
        out
    }

    /// Writes `<source>` and its tests folder. Returns what was created.
    /// An existing source file is never overwritten.
    pub fn create(&self, source: &Path) -> Result<Vec<PathBuf>, String> {
        let io = |path: &Path, e: std::io::Error| format!("can't write {}: {e}", path.display());
        if source.exists() {
            return Err(format!(
                "{} already exists; `cpl new` won't overwrite it",
                source.display()
            ));
        }
        if let Some(parent) = source.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent).map_err(|e| io(parent, e))?;
        }
        std::fs::write(source, self.source(template_for(source).as_deref()))
            .map_err(|e| io(source, e))?;
        let mut created = vec![source.to_path_buf()];
        let tests = tests_dir(source);
        std::fs::create_dir_all(&tests).map_err(|e| io(&tests, e))?;
        created.push(tests.clone());
        for (index, (input, output)) in self.samples.iter().enumerate() {
            let path = tests.join(format!("{}.in", index + 1));
            std::fs::write(&path, input).map_err(|e| io(&path, e))?;
            created.push(path.clone());
            // An interactive sample is a transcript, not an expected output.
            if !output.is_empty() && !self.interactive {
                let path = path.with_extension("out");
                std::fs::write(&path, output).map_err(|e| io(&path, e))?;
                created.push(path);
            }
        }
        Ok(created)
    }
}

/// The project's template: `template.cp` next to the nearest `cp.toml`.
fn template_for(source: &Path) -> Option<String> {
    let start = source
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let mut dir = Some(start.canonicalize().ok()?);
    while let Some(current) = dir {
        if current.join("cp.toml").is_file() {
            return std::fs::read_to_string(current.join("template.cp")).ok();
        }
        dir = current.parent().map(Path::to_path_buf);
    }
    None
}

/// Runs `cpl new`. Returns the exit code.
pub fn run(args: &[String], stdout: &mut dyn Write, stderr: &mut dyn Write) -> u8 {
    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
        let _ = write!(stdout, "{USAGE}");
        return 0;
    }
    let parsed = Args::parse(
        args,
        &[
            "--constraints",
            "--companion",
            "--time-limit",
            "--memory-limit",
        ],
        &[],
    );
    let built = parsed.and_then(|args| {
        let mut scaffold = match args.value("--companion") {
            Some(path) => {
                let text = if path == "-" {
                    let mut text = String::new();
                    std::io::stdin().read_to_string(&mut text).map(|_| text)
                } else {
                    std::fs::read_to_string(path)
                };
                let text = text.map_err(|e| format!("can't read {path}: {e}"))?;
                Scaffold::from_companion(&text)?
            }
            None => Scaffold::default(),
        };
        if let Some(constraints) = args.value("--constraints") {
            scaffold.reads = reads(constraints)?;
        }
        let (time, memory) = (args.number("--time-limit")?, args.number("--memory-limit")?);
        if time.is_some() || memory.is_some() {
            let base = scaffold.limits.unwrap_or_default();
            scaffold.limits = Some(Limits {
                time: time.map_or(base.time, Duration::from_millis),
                memory_kb: memory.map_or(base.memory_kb, |mb| mb * 1024),
            });
        }
        let name = match (args.positional.as_slice(), scaffold.short_name()) {
            ([name], _) => name.clone(),
            ([], Some(name)) => name,
            ([], None) => return Err("`new` needs a name: `cpl new d`".to_string()),
            ([_, extra, ..], _) => {
                return Err(format!("`new` takes one name; `{extra}` is a second"))
            }
        };
        Ok((crate::problem::source_path(&name), scaffold))
    });
    let (source, scaffold) = match built {
        Ok(built) => built,
        Err(message) => {
            let _ = write!(stderr, "error: {message}\n\n{USAGE}");
            return 2;
        }
    };
    match scaffold.create(&source) {
        Ok(created) => {
            let _ = writeln!(stdout, "  created {}", created[0].display());
            let samples = scaffold.samples.len();
            let _ = writeln!(
                stdout,
                "  created {}/ ({})",
                created[1].display(),
                match samples {
                    0 => "no samples yet".to_string(),
                    1 => "1 sample".to_string(),
                    n => format!("{n} samples"),
                }
            );
            0
        }
        Err(message) => {
            let _ = writeln!(stderr, "error: {message}");
            1
        }
    }
}
