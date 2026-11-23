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

use crate::problem::{tests_dir, Limits};
use cpl_diag::Shown;
use cpl_json::{self as json, Value as Json};
use cpl_stress::args::Args;
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
  --listen             wait for the Competitive Companion browser extension
                       to send the problem (or every problem of a contest)
                       and create it; <name> is optional, and a whole
                       contest goes in a folder named after it
  --port N             the port `--listen` waits on (default 10043, one of
                       the ports the extension sends to out of the box)
  --time-limit MS      the problem's time limit in milliseconds
  --memory-limit MB    the problem's memory limit in megabytes
  --json               print what was created as JSON, as `cpl fetch --json`
";

/// One of the ports Competitive Companion posts to by default.
const COMPANION_PORT: u16 = 10043;

/// The body of one HTTP `POST`, answered with an empty `200`.
fn read_post(stream: &mut std::net::TcpStream) -> Result<String, String> {
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .map_err(|e| e.to_string())?;
    let mut bytes = Vec::new();
    let mut chunk = [0u8; 4096];
    let (head_end, length) = loop {
        let n = stream
            .read(&mut chunk)
            .map_err(|e| format!("the request stopped: {e}"))?;
        if n == 0 {
            return Err("the request ended before its headers did".to_string());
        }
        bytes.extend_from_slice(&chunk[..n]);
        if let Some(at) = bytes.windows(4).position(|w| w == b"\r\n\r\n") {
            let head = String::from_utf8_lossy(&bytes[..at]).to_ascii_lowercase();
            let length = head
                .lines()
                .find_map(|line| line.strip_prefix("content-length:"))
                .and_then(|value| value.trim().parse::<usize>().ok())
                .ok_or("the request has no Content-Length")?;
            break (at + 4, length);
        }
        if bytes.len() > 1 << 16 {
            return Err("the request's headers are too long".to_string());
        }
    };
    if length > 64 << 20 {
        return Err("the request is too large to be a problem".to_string());
    }
    while bytes.len() < head_end + length {
        let n = stream
            .read(&mut chunk)
            .map_err(|e| format!("the request stopped: {e}"))?;
        if n == 0 {
            return Err("the request ended before its body did".to_string());
        }
        bytes.extend_from_slice(&chunk[..n]);
    }
    let _ = stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
    String::from_utf8(bytes[head_end..head_end + length].to_vec())
        .map_err(|_| "the request's body is not UTF-8".to_string())
}

/// How many problems the extension is sending: one, or a whole contest.
fn batch_size(text: &str) -> usize {
    json::parse(text)
        .ok()
        .and_then(|problem| problem.get("batch")?.get("size")?.as_u64())
        .map_or(1, |size| size.max(1) as usize)
}

/// Waits on localhost for Competitive Companion and gives each problem
/// it sends to `each`. Only this machine can reach the port.
pub fn listen(
    port: u16,
    stderr: &mut dyn Write,
    each: &mut dyn FnMut(&str) -> Result<(), String>,
) -> Result<(), String> {
    let listener = std::net::TcpListener::bind(("127.0.0.1", port))
        .map_err(|e| format!("can't listen on 127.0.0.1:{port}: {e}"))?;
    listen_on(listener, stderr, each)
}

/// [`listen`] on a socket already bound, so a caller holds its port from
/// bind to accept (a test binds port 0 and nothing can take it between).
pub fn listen_on(
    listener: std::net::TcpListener,
    stderr: &mut dyn Write,
    each: &mut dyn FnMut(&str) -> Result<(), String>,
) -> Result<(), String> {
    let port = listener.local_addr().map_or(0, |a| a.port());
    let _ = writeln!(
        stderr,
        "waiting for Competitive Companion on 127.0.0.1:{port}: open the problem and click the extension"
    );
    let (mut got, mut want) = (0, 1);
    while got < want {
        let (mut stream, _) = listener
            .accept()
            .map_err(|e| format!("can't accept: {e}"))?;
        match read_post(&mut stream) {
            Ok(text) => {
                want = want.max(batch_size(&text));
                got += 1;
                each(&text)?;
            }
            // Something else knocked on the port: keep waiting.
            Err(message) => {
                let _ = writeln!(stderr, "  ignored a request: {message}");
            }
        }
    }
    Ok(())
}

/// What is known about a problem before any code is written.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Scaffold {
    pub title: Option<String>,
    pub url: Option<String>,
    pub limits: Option<Limits>,
    pub interactive: bool,
    /// The input starts with the number of test cases: Competitive
    /// Companion's `"testType": "multiNumber"`. The file gets `@!multitest`.
    pub multitest: bool,
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
            .and_then(Json::as_array)
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
            multitest: problem.get("testType").and_then(Json::as_str) == Some("multiNumber"),
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
        if let Some(judge) = self.url.as_deref().and_then(crate::fetch::judge_profile_of) {
            out.push_str(&format!("// judge: {judge}\n"));
        }
        if let Some(limits) = self.limits {
            out.push_str(&limits.header());
            out.push('\n');
        }
        if self.interactive {
            out.push_str(
                "// interactive: an `@interactor fn judge(mut sol: Interactor)` here plays the judge \
                 on each sample; `cpl run` and `cpl test` use it\n",
            );
        }
        if !out.is_empty() {
            out.push('\n');
        }
        let template = template.map(str::trim_end);
        if self.multitest && !template.is_some_and(|t| t.contains("@!multitest")) {
            out.push_str("@!multitest\n");
        }
        for line in &self.reads {
            out.push_str(line);
            out.push('\n');
        }
        match template {
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
        let io = |path: &Path, e: std::io::Error| format!("can't write {}: {e}", path.shown());
        if source.exists() {
            return Err(format!(
                "{} already exists; `cpl new` won't overwrite it",
                source.shown()
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
            "--port",
        ],
        &["--listen", "--json"],
    );
    if let Ok(args) = &parsed {
        if args.has("--listen") {
            return run_listen(args, stdout, stderr);
        }
    }
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
            ([], None) => return Err(
                    "`new` needs a name: `cpl new d`, or `cpl new --listen` to import from Competitive Companion"
                        .to_string(),
                ),
            ([_, extra, ..], _) => {
                return Err(format!("`new` takes one name; `{extra}` is a second"))
            }
        };
        Ok((crate::problem::source_path(&name), scaffold))
    });
    let (source, scaffold) = match built {
        Ok(built) => built,
        Err(message) => {
            let _ = write!(stderr, "cpl: {message}\n\n{USAGE}");
            return 2;
        }
    };
    let json = args.iter().any(|arg| arg == "--json");
    report(&scaffold, &source, json, stdout, stderr)
}

/// `cpl new --listen`: every problem the extension sends becomes a file.
fn run_listen(args: &Args, stdout: &mut dyn Write, stderr: &mut dyn Write) -> u8 {
    let usage = |stderr: &mut dyn Write, message: String| {
        let _ = write!(stderr, "cpl: {message}\n\n{USAGE}");
        2
    };
    if args.value("--companion").is_some() || args.value("--constraints").is_some() {
        return usage(
            stderr,
            "`--listen` gets the problem from the browser: drop `--companion` and `--constraints`"
                .into(),
        );
    }
    let port = match args.number("--port") {
        Ok(None) => COMPANION_PORT,
        Ok(Some(port)) if (1..=u64::from(u16::MAX)).contains(&port) => port as u16,
        Ok(Some(port)) => return usage(stderr, format!("`--port {port}` is not a port")),
        Err(message) => return usage(stderr, message),
    };
    if args.positional.len() > 1 {
        return usage(
            stderr,
            format!("`new` takes one name; `{}` is a second", args.positional[1]),
        );
    }
    let mut named = args.positional.first().cloned();
    let mut code = 0;
    let heard = listen(port, stderr, &mut |text| {
        let scaffold = Scaffold::from_companion(text)?;
        // The name given is for the first problem; the rest of a contest
        // are named after their own URLs.
        let source = match named.take() {
            Some(name) => crate::problem::source_path(&name),
            None => {
                let cwd = std::env::current_dir().unwrap_or_default();
                contest_source(&scaffold, batch_size(text), &cwd).ok_or(
                    "the problem has no URL to name it after: give a name, `cpl new d --listen`",
                )?
            }
        };
        let mut errors = Vec::new();
        code = code.max(report(
            &scaffold,
            &source,
            args.has("--json"),
            stdout,
            &mut errors,
        ));
        if !errors.is_empty() {
            return Err(String::from_utf8_lossy(&errors)
                .trim_start_matches("cpl: ")
                .trim_end()
                .to_string());
        }
        Ok(())
    });
    match heard {
        Ok(()) => code,
        Err(message) => {
            let _ = writeln!(stderr, "cpl: {message}");
            1
        }
    }
}

/// Where a problem the extension sent goes when no name was given: named
/// after its title, in a folder named after its contest when the extension
/// sends a whole contest, unless `cwd` already is that folder.
pub(crate) fn contest_source(scaffold: &Scaffold, batch: usize, cwd: &Path) -> Option<PathBuf> {
    let file = crate::problem::source_path(&scaffold.short_name()?);
    let contest = scaffold
        .url
        .as_deref()
        .and_then(crate::fetch::contest_of)
        .filter(|contest| batch > 1 && cwd.file_name().and_then(|n| n.to_str()) != Some(contest));
    Some(match contest {
        Some(contest) => Path::new(&contest).join(file),
        None => file,
    })
}

/// Creates the problem and says what was made.
fn report(
    scaffold: &Scaffold,
    source: &Path,
    json: bool,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> u8 {
    match scaffold.create(source) {
        Ok(created) => {
            let samples = scaffold.samples.len();
            if json {
                let _ = writeln!(
                    stdout,
                    "{{\"path\":{},\"created\":true,\"tests\":{},\"samples\":{samples}}}",
                    cpl_json::escape_string(&created[0].shown()),
                    cpl_json::escape_string(&created[1].shown()),
                );
                return 0;
            }
            let _ = writeln!(stdout, "  created {}", created[0].shown());
            let _ = writeln!(
                stdout,
                "  created {}/ ({})",
                created[1].shown(),
                match samples {
                    0 => "no samples yet".to_string(),
                    1 => "1 sample".to_string(),
                    n => format!("{n} samples"),
                }
            );
            let _ = writeln!(
                stdout,
                "  next: write it; `cpl test {}` runs the samples",
                created[0].shown()
            );
            0
        }
        Err(message) => {
            let _ = writeln!(stderr, "cpl: {message}");
            1
        }
    }
}
