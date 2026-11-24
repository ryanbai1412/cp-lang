//! `cpl fetch`: a problem's samples from its page on the judge.
//!
//! ```text
//! cpl fetch https://atcoder.jp/contests/abc300/tasks/abc300_a
//! ```
//!
//! writes `a.tests/1.in`, `1.out`, ... and, if there is no `a.cp` yet, the
//! same file `cpl new` makes, headed with the problem's title and limits.
//! Codeforces turns scripts away, so `--html` reads a page saved from the
//! browser instead (decisions.md 250).

use crate::new::Scaffold;
use crate::problem::{source_path, tests_dir, Limits};
use cpl_diag::Shown;
use cpl_stress::args::Args;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

pub const USAGE: &str = "\
usage: cpl fetch <url> [name] [options]

Downloads a problem's samples into <name>.tests/ (1.in, 1.out, ...) and,
if <name>.cp doesn't exist, creates it with the title and limits. Knows
Codeforces, AtCoder and CSES problem pages; for other judges, use `cpl new
--listen` with the Competitive Companion browser extension. <name> is the
problem's letter unless given: `a` for `A. Watermelon`, `cses1068` for
CSES task 1068.

A contest's page (`atcoder.jp/contests/abc300`, `codeforces.com/contest/1950`)
fetches every problem it lists into a folder named after the contest, or
[name]: abc300/a.cp, abc300/b.cp, ...

options:
  --html FILE   read the page from FILE, saved from the browser, instead of
                downloading it (Codeforces refuses scripts)
  --json        print what was written as JSON
";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Judge {
    Codeforces,
    AtCoder,
    Cses,
}

/// The judge whose problem page `url` is.
pub fn judge_of(url: &str) -> Option<Judge> {
    let rest = url.split_once("://").map_or(url, |(_, rest)| rest);
    let host = rest.split(['/', '?', '#']).next()?.to_ascii_lowercase();
    let is = |domain: &str| host == domain || host.ends_with(&format!(".{domain}"));
    if is("codeforces.com") {
        Some(Judge::Codeforces)
    } else if is("atcoder.jp") {
        Some(Judge::AtCoder)
    } else if is("cses.fi") {
        Some(Judge::Cses)
    } else {
        None
    }
}

/// The built-in judge profile for the judge whose problem page `url` is,
/// for the `// judge:` line of a new file.
pub fn judge_profile_of(url: &str) -> Option<&'static str> {
    match judge_of(url)? {
        Judge::Codeforces => Some("codeforces"),
        Judge::AtCoder => Some("atcoder"),
        Judge::Cses => Some("cses"),
    }
}

/// The contest a problem page belongs to, as its judge names it in the URL:
/// `1950` for `codeforces.com/contest/1950/problem/A`, `abc300` for
/// `atcoder.jp/contests/abc300/tasks/abc300_a`.
pub fn contest_of(url: &str) -> Option<String> {
    let rest = url.split_once("://").map_or(url, |(_, rest)| rest);
    let path: Vec<&str> = rest.split(['?', '#']).next()?.split('/').skip(1).collect();
    let key = match judge_of(url)? {
        Judge::Codeforces => ["contest", "gym"].as_slice(),
        Judge::AtCoder => ["contests"].as_slice(),
        Judge::Cses => return None,
    };
    let at = path.iter().position(|part| key.contains(part))?;
    let id = path.get(at + 1).filter(|id| !id.is_empty())?;
    id.chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        .then(|| id.to_string())
}

/// The contest whose own page `url` is, rather than one of its problems':
/// `abc300` for `atcoder.jp/contests/abc300` and its `/tasks` list, `1950`
/// for `codeforces.com/contest/1950`.
pub fn contest_page(url: &str) -> Option<String> {
    let contest = contest_of(url)?;
    let rest = url.split_once("://").map_or(url, |(_, rest)| rest);
    let path: Vec<&str> = rest.split(['?', '#']).next()?.split('/').skip(1).collect();
    let after = path.iter().position(|part| *part == contest)? + 1;
    let tail: Vec<&str> = path[after..]
        .iter()
        .copied()
        .filter(|p| !p.is_empty())
        .collect();
    let list = match judge_of(url)? {
        Judge::AtCoder => "tasks",
        _ => "problems",
    };
    matches!(tail.as_slice(), [] | [_]).then_some(())?;
    (tail.is_empty() || tail == [list]).then_some(contest)
}

/// The origin and the path up to the contest id of a contest's url:
/// (`https://atcoder.jp`, `/contests/abc300`).
fn contest_root(url: &str, contest: &str) -> Option<(String, String)> {
    let rest = url.split_once("://").map_or(url, |(_, rest)| rest);
    let rest = rest.split(['?', '#']).next()?;
    let (host, path) = rest.split_once('/')?;
    let end = path.split('/').position(|part| part == contest)?;
    let prefix: Vec<&str> = path.split('/').take(end + 1).collect();
    Some((format!("https://{host}"), format!("/{}", prefix.join("/"))))
}

/// The page that lists a contest's problems.
fn contest_list_url(judge: Judge, url: &str, contest: &str) -> Option<String> {
    let (origin, root) = contest_root(url, contest)?;
    Some(match judge {
        Judge::AtCoder => format!("{origin}{root}/tasks"),
        _ => format!("{origin}{root}"),
    })
}

/// The problem pages a contest's page links to, in its order, each once.
pub fn problem_urls(judge: Judge, url: &str, contest: &str, html: &str) -> Vec<String> {
    let Some((origin, root)) = contest_root(url, contest) else {
        return Vec::new();
    };
    let marker = match judge {
        Judge::AtCoder => format!("href=\"{root}/tasks/"),
        Judge::Codeforces => format!("href=\"{root}/problem/"),
        Judge::Cses => return Vec::new(),
    };
    let mut found: Vec<String> = Vec::new();
    for piece in html.split(marker.as_str()).skip(1) {
        let id: String = piece
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
            .collect();
        if id.is_empty() || !piece[id.len()..].starts_with('"') {
            continue;
        }
        let page = match judge {
            Judge::AtCoder => format!("{origin}{root}/tasks/{id}"),
            _ => format!("{origin}{root}/problem/{id}"),
        };
        if !found.contains(&page) {
            found.push(page);
        }
    }
    found
}

/// The title, limits and samples on a problem page.
pub fn parse(judge: Judge, url: &str, html: &str) -> Result<Scaffold, String> {
    let (title, limits, samples) = match judge {
        Judge::Codeforces => codeforces(html),
        Judge::AtCoder => atcoder(html),
        Judge::Cses => cses(html),
    };
    if samples.is_empty() {
        return Err(format!(
            "found no samples on the page for {url}; is it a problem page?"
        ));
    }
    Ok(Scaffold {
        title,
        url: Some(url.to_string()),
        limits,
        samples,
        multitest: judge == Judge::Codeforces && counts_test_cases(html),
        any_answer: accepts_any_answer(judge, html),
        ..Scaffold::default()
    })
}

/// A Codeforces input section that opens with the number of test cases:
/// "The first line contains a single integer t ... the number of test
/// cases". Only the first sentence or so is read.
fn counts_test_cases(html: &str) -> bool {
    let Some(rest) = after(html, "class=\"input-specification\"") else {
        return false;
    };
    let first = text(prefix(rest, 800)).to_ascii_lowercase();
    let first = &first[..first.find(". ").map_or(first.len(), |end| end + 1)];
    // "the number of test cases" (the first line holds t) or "each test
    // contains multiple test cases" (the sentence before it says so).
    ["number of test", "multiple test", "several test"]
        .iter()
        .any(|phrase| first.contains(phrase))
}

/// An output section that accepts any of several answers: "If there are
/// multiple answers, print any of them".
fn accepts_any_answer(judge: Judge, html: &str) -> bool {
    let (start, end) = match judge {
        Judge::Codeforces => ("class=\"output-specification\"", "class=\"sample-tests\""),
        Judge::AtCoder => ("<h3>Output</h3>", "<h3>Sample Input"),
        Judge::Cses => ("<h1 id=\"output\">", "<h1"),
    };
    let english = html
        .find("<span class=\"lang-en\">")
        .map_or(html, |at| &html[at..]);
    let Some(rest) = after(english, start) else {
        return false;
    };
    let section = text(&rest[..rest.find(end).unwrap_or(rest.len())]).to_ascii_lowercase();
    let section = section.split_whitespace().collect::<Vec<_>>().join(" ");
    [
        "print any",
        "output any",
        "any of them",
        "any one of them",
        "any valid",
        "any such",
        "multiple answers",
        "multiple possible answers",
        "multiple solutions",
        "several answers",
        "several possible answers",
    ]
    .iter()
    .any(|phrase| section.contains(phrase))
}

type Page = (Option<String>, Option<Limits>, Vec<(String, String)>);

fn codeforces(html: &str) -> Page {
    let inputs = pres_after(html, "class=\"input\"");
    let outputs = pres_after(html, "class=\"output\"");
    let samples = inputs
        .iter()
        .zip(outputs.iter().map(Some).chain(std::iter::repeat(None)))
        .map(|(input, output)| (sample(input), output.map_or(String::new(), |o| sample(o))))
        .collect();
    let statement = html
        .find("class=\"problem-statement\"")
        .map_or(html, |at| &html[at..]);
    let title = after(statement, "class=\"title\">")
        .map(|rest| {
            decode(&rest[..rest.find('<').unwrap_or(rest.len())])
                .trim()
                .to_string()
        })
        .filter(|title| !title.is_empty());
    let header = after(statement, "class=\"time-limit\"")
        .map(|rest| text(prefix(rest, 600)))
        .unwrap_or_default();
    let limits = limits(
        number_before(&header, "second"),
        number_before(&header, "megabyte").map(|mb| mb * 1024.0),
    );
    (title, limits, samples)
}

fn atcoder(html: &str) -> Page {
    let english = html
        .find("<span class=\"lang-en\">")
        .map_or(html, |at| &html[at..]);
    let mut samples = Vec::new();
    for n in 1.. {
        let pre = |kind: &str| {
            after(english, &format!("<h3>Sample {kind} {n}</h3>"))
                .and_then(|rest| pres_after(rest, "").into_iter().next())
                .map(sample)
        };
        let Some(input) = pre("Input") else { break };
        samples.push((input, pre("Output").unwrap_or_default()));
    }
    let title = after(html, "<span class=\"h2\">")
        .map(|rest| {
            decode(&rest[..rest.find('<').unwrap_or(rest.len())])
                .trim()
                .to_string()
        })
        .filter(|title| !title.is_empty());
    let line = after(html, "Time Limit:")
        .map(|rest| rest[..rest.find('<').unwrap_or(rest.len())].to_string())
        .unwrap_or_default();
    let memory = number_before(&line, "MiB")
        .map(|mib| mib * 1024.0)
        .or_else(|| number_before(&line, "MB").map(|mb| mb * 1000.0));
    (title, limits(number_before(&line, "sec"), memory), samples)
}

fn cses(html: &str) -> Page {
    let inputs = pres_after(html, "<p>Input:</p>");
    let outputs = pres_after(html, "<p>Output:</p>");
    let samples = inputs
        .iter()
        .zip(outputs.iter().map(Some).chain(std::iter::repeat(None)))
        .map(|(input, output)| (sample(input), output.map_or(String::new(), |o| sample(o))))
        .collect();
    let title = after(html, "<h1>")
        .map(|rest| {
            decode(&rest[..rest.find('<').unwrap_or(rest.len())])
                .trim()
                .to_string()
        })
        .filter(|title| !title.is_empty());
    // `Time limit: 1.00 s`, `Memory limit: 512 MB`.
    let header = after(html, "class=\"task-constraints\"")
        .map(|rest| text(prefix(rest, 300)))
        .unwrap_or_default();
    let limits = limits(
        number_before(&header, " s\n").or_else(|| number_before(&header, " s")),
        number_before(&header, "MB").map(|mb| mb * 1024.0),
    );
    (title, limits, samples)
}

/// `cses1068` for `cses.fi/problemset/task/1068`: CSES titles have no letter.
fn cses_name(url: &str) -> Option<String> {
    let id = url
        .split(['?', '#'])
        .next()?
        .trim_end_matches('/')
        .rsplit('/')
        .next()?;
    (!id.is_empty() && id.chars().all(|c| c.is_ascii_digit())).then(|| format!("cses{id}"))
}

fn limits(seconds: Option<f64>, memory_kb: Option<f64>) -> Option<Limits> {
    if seconds.is_none() && memory_kb.is_none() {
        return None;
    }
    let defaults = Limits::default();
    Some(Limits {
        time: seconds.map_or(defaults.time, Duration::from_secs_f64),
        memory_kb: memory_kb.map_or(defaults.memory_kb, |kb| kb as u64),
    })
}

/// At most the first `bytes` of `html`, cut on a character boundary.
fn prefix(html: &str, bytes: usize) -> &str {
    let mut end = bytes.min(html.len());
    while !html.is_char_boundary(end) {
        end -= 1;
    }
    &html[..end]
}

fn after<'a>(html: &'a str, marker: &str) -> Option<&'a str> {
    html.find(marker).map(|at| &html[at + marker.len()..])
}

/// The inside of the first `<pre>` after each `marker`.
fn pres_after<'a>(html: &'a str, marker: &str) -> Vec<&'a str> {
    let mut found = Vec::new();
    let mut rest = html;
    while let Some(next) = after(rest, marker) {
        let Some(open) = next.find("<pre") else { break };
        let Some(start) = next[open..].find('>').map(|i| open + i + 1) else {
            break;
        };
        let Some(len) = next[start..].find("</pre>") else {
            break;
        };
        found.push(&next[start..start + len]);
        rest = &next[start + len..];
        if marker.is_empty() {
            break;
        }
    }
    found
}

/// A sample's text: one line per line of the page, ending in a newline.
fn sample(pre: &str) -> String {
    let text = text(pre);
    let lines: Vec<&str> = text.lines().map(str::trim_end).collect();
    let start = lines
        .iter()
        .position(|l| !l.is_empty())
        .unwrap_or(lines.len());
    let end = lines
        .iter()
        .rposition(|l| !l.is_empty())
        .map_or(start, |i| i + 1);
    lines[start..end].iter().map(|l| format!("{l}\n")).collect()
}

/// The text of some HTML: a `<br>` or the end of a `<div>` (Codeforces puts
/// each sample line in one) is a new line, other tags go.
fn text(html: &str) -> String {
    let mut out = String::new();
    let mut rest = html;
    while let Some(open) = rest.find('<') {
        out.push_str(&rest[..open]);
        let close = rest[open..].find('>').map_or(rest.len(), |i| open + i + 1);
        let tag = rest[open..close].to_ascii_lowercase();
        if tag.starts_with("<br") || tag == "</div>" {
            out.push('\n');
        }
        rest = &rest[close..];
    }
    out.push_str(rest);
    decode(&out)
}

fn decode(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
}

/// The number written just before `unit`: `2` in `2 seconds`.
fn number_before(text: &str, unit: &str) -> Option<f64> {
    let head = text[..text.find(unit)?].trim_end();
    let start = head
        .char_indices()
        .rfind(|&(_, c)| !(c.is_ascii_digit() || c == '.'))
        .map_or(0, |(i, c)| i + c.len_utf8());
    head[start..].parse().ok()
}

pub fn run(args: &[String], stdout: &mut dyn Write, stderr: &mut dyn Write) -> u8 {
    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
        let _ = write!(stdout, "{USAGE}");
        return 0;
    }
    // `-o NAME` is the guess other tools teach; say where the name goes.
    if let Some(at) = args
        .iter()
        .position(|arg| matches!(arg.as_str(), "-o" | "--out" | "--output"))
    {
        let name = args
            .get(at + 1)
            .map_or("a", |name| name.trim_end_matches(".cp"));
        let url = args
            .iter()
            .find(|arg| judge_of(arg).is_some())
            .map_or("<url>", String::as_str);
        let _ = writeln!(
            stderr,
            "cpl: `fetch` takes the name as its second argument, without `{}` or `.cp`: `cpl fetch {url} {name}`",
            args[at]
        );
        return 2;
    }
    let parsed = Args::parse("fetch", args, &["--html"], &["--json"]).and_then(|args| {
        let (url, name) = match args.positional.as_slice() {
            [url] => (url.clone(), None),
            [url, name] => (url.clone(), Some(name.clone())),
            [] => return Err("`fetch` needs a problem page: `cpl fetch <url>`".to_string()),
            [_, _, extra, ..] => {
                return Err(format!(
                    "`fetch` takes a url and a name; `{extra}` is a third"
                ))
            }
        };
        let judge = judge_of(&url).ok_or_else(|| {
            format!(
                "`{url}` is not a Codeforces, AtCoder or CSES page; for other judges, \
                 `cpl new --listen` takes the problem from Competitive Companion"
            )
        })?;
        Ok((args, url, name, judge))
    });
    let (args, url, name, judge) = match parsed {
        Ok(parsed) => parsed,
        Err(message) => {
            let _ = write!(stderr, "cpl: {message}\n\n{USAGE}");
            return 2;
        }
    };
    if let Some(contest) = contest_page(&url) {
        return fetch_contest(judge, &url, &contest, name, &args, stdout, stderr);
    }
    let fetched = match args.value("--html") {
        Some(path) => std::fs::read_to_string(path).map_err(|e| format!("can't read {path}: {e}")),
        None => download(&url),
    }
    .and_then(|html| parse(judge, &url, &html))
    .and_then(|scaffold| {
        let name = name
            .or_else(|| (judge == Judge::Cses).then(|| cses_name(&url)).flatten())
            .or_else(|| scaffold.short_name())
            .ok_or_else(|| {
                format!("the page for {url} has no title; name the problem: `cpl fetch {url} d`")
            })?;
        let source = source_path(&name);
        let created = write(&scaffold, &source)?;
        Ok((scaffold, source, created))
    });
    let (scaffold, source, created) = match fetched {
        Ok(fetched) => fetched,
        Err(message) => {
            let _ = writeln!(stderr, "cpl: {message}");
            return 1;
        }
    };
    let tests = tests_dir(&source);
    let samples = scaffold.samples.len();
    if args.has("--json") {
        let json = cpl_json::write_with(|w| {
            w.object()
                .key("path")
                .str(&source.shown())
                .key("created")
                .bool(created);
            w.key("tests")
                .str(&tests.shown())
                .key("sample_count")
                .int(samples)
                .end();
        });
        let _ = writeln!(stdout, "{json}");
        return 0;
    }
    let count = match samples {
        1 => "1 sample".to_string(),
        n => format!("{n} samples"),
    };
    if created {
        let _ = writeln!(stdout, "  created {}", source.shown());
        let _ = writeln!(stdout, "  created {}/ ({count})", tests.shown());
        let _ = writeln!(
            stdout,
            "  next: write it; `cpl test {}` runs the samples",
            source.shown()
        );
    } else {
        let _ = writeln!(stdout, "  wrote {count} to {}/", tests.shown());
    }
    0
}

/// `cpl fetch` on a contest's page: every problem it lists, each as `cpl
/// fetch` on that problem's page makes it, in one folder.
fn fetch_contest(
    judge: Judge,
    url: &str,
    contest: &str,
    name: Option<String>,
    args: &Args,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> u8 {
    let listed = contest_list_url(judge, url, contest)
        .ok_or_else(|| format!("can't tell the contest's page from {url}"))
        .and_then(|list| match args.value("--html") {
            Some(path) => {
                std::fs::read_to_string(path).map_err(|e| format!("can't read {path}: {e}"))
            }
            None => download(&list),
        })
        .map(|html| problem_urls(judge, url, contest, &html));
    let problems = match listed {
        Ok(problems) if problems.is_empty() => {
            let _ = writeln!(stderr, "cpl: found no problems on the page for {url}");
            return 1;
        }
        Ok(problems) => problems,
        Err(message) => {
            let _ = writeln!(stderr, "cpl: {message}");
            return 1;
        }
    };
    let folder_name = name.unwrap_or_else(|| contest.to_string());
    let cwd = std::env::current_dir().unwrap_or_default();
    let folder = if cwd.file_name().and_then(|n| n.to_str()) == Some(folder_name.as_str()) {
        PathBuf::new()
    } else {
        PathBuf::from(&folder_name)
    };
    let json = args.has("--json");
    let (mut rows, mut failed, mut first) = (Vec::new(), 0, None);
    for (index, page) in problems.iter().enumerate() {
        if index > 0 {
            // One page a second, as a person clicking through would.
            std::thread::sleep(Duration::from_secs(1));
        }
        let fetched = download(page)
            .and_then(|html| parse(judge, page, &html))
            .and_then(|scaffold| {
                let id = page.rsplit('/').next().unwrap_or("p");
                let short = scaffold.short_name().unwrap_or_else(|| id.to_string());
                let source = folder.join(source_path(&short));
                if !folder.as_os_str().is_empty() {
                    std::fs::create_dir_all(&folder)
                        .map_err(|e| format!("can't create {}: {e}", folder.shown()))?;
                }
                let created = write(&scaffold, &source)?;
                Ok((source, created, scaffold.samples.len()))
            });
        match fetched {
            Ok((source, created, samples)) => {
                let tests = tests_dir(&source);
                first.get_or_insert_with(|| source.clone());
                if json {
                    rows.push(cpl_json::write_with(|w| {
                        w.object()
                            .key("url")
                            .str(page)
                            .key("path")
                            .str(&source.shown());
                        w.key("created")
                            .bool(created)
                            .key("tests")
                            .str(&tests.shown());
                        w.key("sample_count").int(samples).end();
                    }));
                } else {
                    let count = match samples {
                        1 => "1 sample".to_string(),
                        n => format!("{n} samples"),
                    };
                    let did = if created { "created" } else { "updated" };
                    let _ = writeln!(stdout, "  {did} {} ({count})", source.shown());
                }
            }
            Err(message) => {
                failed += 1;
                if json {
                    rows.push(cpl_json::write_with(|w| {
                        w.object()
                            .key("url")
                            .str(page)
                            .key("error")
                            .str(&message)
                            .end();
                    }));
                } else {
                    let _ = writeln!(stderr, "cpl: {page}: {message}");
                }
            }
        }
    }
    if json {
        let json = cpl_json::write_with(|w| {
            w.object()
                .key("contest")
                .str(contest)
                .key("folder")
                .str(&folder.shown());
            w.key("problems").array();
            for row in &rows {
                w.json(row);
            }
            w.end().end();
        });
        let _ = writeln!(stdout, "{json}");
    } else if let Some(first) = first {
        let _ = writeln!(
            stdout,
            "  next: write them; `cpl test {}` runs its samples",
            first.shown()
        );
    }
    u8::from(failed > 0)
}

fn download(url: &str) -> Result<String, String> {
    let out = Command::new("curl")
        .args(["-fsSL", "--retry", "2", "--connect-timeout", "20"])
        .args([
            "--proto",
            "=https",
            "--proto-redir",
            "=https",
            "-A",
            "cpl fetch",
        ])
        .arg(url)
        .output()
        .map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => "`cpl fetch` needs curl on the PATH".to_string(),
            _ => format!("could not run curl: {e}"),
        })?;
    if !out.status.success() {
        return Err(format!(
            "could not download {url}: {}; import it in one click with `cpl new --listen` and \
             the Competitive Companion browser extension, or save the page from the browser \
             and pass it with `--html page.html`",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Writes the samples, and the source if it is new; true if it was. A
/// sample file already there must say the same, so a stress case or a
/// hand-written test is never overwritten.
fn write(scaffold: &Scaffold, source: &Path) -> Result<bool, String> {
    if !source.exists() {
        scaffold.create(source)?;
        return Ok(true);
    }
    let tests = tests_dir(source);
    let mut files: Vec<(PathBuf, &str)> = Vec::new();
    for (index, (input, output)) in scaffold.samples.iter().enumerate() {
        let path = tests.join(format!("{}.in", index + 1));
        if !output.is_empty() {
            files.push((path.with_extension("out"), output));
        }
        files.push((path, input));
    }
    for (path, text) in &files {
        if std::fs::read_to_string(path).is_ok_and(|old| old != *text) {
            return Err(format!(
                "{} is already there and says something else; move it, or fetch into \
                 another name",
                path.shown()
            ));
        }
    }
    let io = |path: &Path, e: std::io::Error| format!("can't write {}: {e}", path.shown());
    std::fs::create_dir_all(&tests).map_err(|e| io(&tests, e))?;
    for (path, text) in files {
        std::fs::write(&path, text).map_err(|e| io(&path, e))?;
    }
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_contest_page_lists_its_problems_once_each() {
        for (url, contest) in [
            ("https://atcoder.jp/contests/abc300", Some("abc300")),
            ("https://atcoder.jp/contests/abc300/tasks", Some("abc300")),
            ("atcoder.jp/contests/abc300/", Some("abc300")),
            ("https://codeforces.com/contest/1950", Some("1950")),
            ("https://codeforces.com/gym/104/problems", Some("104")),
            ("https://atcoder.jp/contests/abc300/tasks/abc300_a", None),
            ("https://codeforces.com/contest/1950/problem/A", None),
            ("https://atcoder.jp/contests/abc300/standings", None),
            ("https://cses.fi/problemset/task/1068", None),
        ] {
            assert_eq!(contest_page(url).as_deref(), contest, "{url}");
        }
        let atcoder = r#"<td><a href="/contests/abc300/tasks/abc300_a">A</a></td>
            <td><a href="/contests/abc300/tasks/abc300_a">N-choice question</a></td>
            <td><a href="/contests/abc300/tasks/abc300_h">Ex</a></td>
            <a href="/contests/abc300/tasks/abc300_a?lang=ja">ja</a>"#;
        assert_eq!(
            problem_urls(
                Judge::AtCoder,
                "atcoder.jp/contests/abc300",
                "abc300",
                atcoder
            ),
            [
                "https://atcoder.jp/contests/abc300/tasks/abc300_a",
                "https://atcoder.jp/contests/abc300/tasks/abc300_h"
            ]
        );
        let gym = r#"<a href="/gym/104/problem/A">A</a><a href="/gym/104/problem/B1">B1</a>
            <a href="/contest/104/problem/C">elsewhere</a>"#;
        assert_eq!(
            problem_urls(
                Judge::Codeforces,
                "https://codeforces.com/gym/104",
                "104",
                gym
            ),
            [
                "https://codeforces.com/gym/104/problem/A",
                "https://codeforces.com/gym/104/problem/B1"
            ]
        );
        assert_eq!(
            contest_list_url(
                Judge::AtCoder,
                "https://atcoder.jp/contests/abc300",
                "abc300"
            )
            .as_deref(),
            Some("https://atcoder.jp/contests/abc300/tasks")
        );
    }

    #[test]
    fn a_page_is_known_by_its_host() {
        let cf = "https://codeforces.com/contest/4/problem/A";
        assert_eq!(judge_of(cf), Some(Judge::Codeforces));
        assert_eq!(
            judge_of("https://mirror.codeforces.com/gym/1/problem/B"),
            Some(Judge::Codeforces)
        );
        assert_eq!(
            judge_of("atcoder.jp/contests/abc300/tasks/abc300_a"),
            Some(Judge::AtCoder)
        );
        assert_eq!(judge_of("https://notcodeforces.com/x"), None);
        assert_eq!(judge_of("https://example.com/?u=codeforces.com"), None);
        for (url, contest) in [
            (cf, Some("4")),
            ("https://codeforces.com/gym/104/problem/B", Some("104")),
            ("https://codeforces.com/problemset/problem/1/D", None),
            (
                "atcoder.jp/contests/abc300/tasks/abc300_a?lang=en",
                Some("abc300"),
            ),
            ("https://example.com/contest/7", None),
        ] {
            assert_eq!(contest_of(url).as_deref(), contest, "{url}");
        }
        for (url, judge) in [
            (cf, Some("codeforces")),
            ("atcoder.jp/contests/abc300/tasks/abc300_a", Some("atcoder")),
            ("https://cses.fi/problemset/task/1068", Some("cses")),
            ("https://example.com/problem/1", None),
        ] {
            assert_eq!(judge_profile_of(url), judge, "{url}");
            if let Some(judge) = judge {
                assert!(cpl_judge::builtin(judge).is_some(), "{judge}");
            }
        }
    }

    /// Codeforces' two sample layouts: one `<div>` per line, and `<br />`.
    #[test]
    fn codeforces_samples_title_and_limits() {
        let html = r#"<div class="problem-statement"><div class="header"><div class="title">A. Watermelon</div><div class="time-limit"><div class="property-title">time limit per test</div>1 second</div><div class="memory-limit"><div class="property-title">memory limit per test</div>64 megabytes</div></div>
<div class="input-specification"><div class="section-title">Input</div><p>w</p></div>
<div class="sample-tests"><div class="sample-test"><div class="input"><div class="title">Input</div><pre>
<div class="test-example-line test-example-line-even test-example-line-0">2 3</div><div class="test-example-line test-example-line-odd test-example-line-1">a &lt; b</div></pre></div><div class="output"><div class="title">Output</div><pre>
YES
</pre></div><div class="input"><div class="title">Input</div><pre>8<br />1 2<br /></pre></div><div class="output"><div class="title">Output</div><pre>NO<br /></pre></div></div></div></div>"#;
        let page = parse(Judge::Codeforces, "u", html).unwrap();
        assert!(!page.multitest);
        let multi = html.replace(
            "<p>w</p>",
            "<p>The first line contains one integer <span>t</span> (1 &le; t &le; 10<sup>4</sup>) &mdash; the number of test cases. Each test case is one line.</p>",
        );
        assert!(parse(Judge::Codeforces, "u", &multi).unwrap().multitest);
        let stated = html.replace(
            "<p>w</p>",
            "<p>Each test contains multiple test cases. The first line contains the number of test cases <span>t</span>.</p>",
        );
        assert!(parse(Judge::Codeforces, "u", &stated).unwrap().multitest);
        let later = html.replace(
            "<p>w</p>",
            "<p>The first line contains n. The answer is the number of test cases you pass.</p>",
        );
        assert!(!parse(Judge::Codeforces, "u", &later).unwrap().multitest);
        assert!(!page.any_answer);
        let any = html.replace(
            "<div class=\"sample-tests\">",
            "<div class=\"output-specification\"><div class=\"section-title\">Output</div><p>Print the edges. If there are multiple answers, print any of them.</p></div><div class=\"sample-tests\">",
        );
        assert!(parse(Judge::Codeforces, "u", &any).unwrap().any_answer);
        let story = html.replace("<p>w</p>", "<p>Print any of them.</p>");
        assert!(
            !parse(Judge::Codeforces, "u", &story).unwrap().any_answer,
            "the input section says nothing of the output"
        );
        assert_eq!(prefix("a\u{e9}b", 2), "a");
        assert_eq!(page.title.as_deref(), Some("A. Watermelon"));
        assert_eq!(page.short_name().as_deref(), Some("a"));
        let limits = page.limits.unwrap();
        assert_eq!(
            (limits.time, limits.memory_kb),
            (Duration::from_secs(1), 64 * 1024)
        );
        assert_eq!(
            page.samples,
            [
                ("2 3\na < b\n".to_string(), "YES\n".to_string()),
                ("8\n1 2\n".to_string(), "NO\n".to_string()),
            ]
        );
    }

    /// Cut from the CSES page for task 1640, as served.
    #[test]
    fn cses_samples_title_limits_and_any_answer() {
        let html = "<div class=\"title-block\">\n                <h1>Sum of Two Values</h1>\n</div>\n\
            <ul class=\"task-constraints\">\n<li><b>Time limit:</b> 1.00 s</li>\n<li><b>Memory limit:</b> 512 MB</li>\n</ul>\n\
            <div class=\"md\"><p>You are given an array.</p>\n<h1 id=\"input\">Input</h1>\n<p>The first input line has two integers.</p>\n\
            <h1 id=\"output\">Output</h1>\n<p>Print two integers. If there are several solutions, you may print any of them.</p>\n\
            <h1 id=\"constraints\">Constraints</h1>\n<h1 id=\"example\">Example</h1>\n\
            <p>Input:</p>\n<pre>4 8\n2 7 5 1\n</pre>\n<p>Output:</p>\n<pre>2 4\n</pre></div>";
        let url = "https://cses.fi/problemset/task/1640";
        let page = parse(Judge::Cses, url, html).unwrap();
        assert_eq!(page.title.as_deref(), Some("Sum of Two Values"));
        let limits = page.limits.unwrap();
        assert_eq!(
            (limits.time, limits.memory_kb),
            (Duration::from_secs(1), 512 * 1024)
        );
        assert_eq!(
            page.samples,
            [("4 8\n2 7 5 1\n".to_string(), "2 4\n".to_string())]
        );
        assert!(page.any_answer && !page.multitest);
        assert_eq!(judge_of(url), Some(Judge::Cses));
        assert_eq!(contest_of(url), None);
        assert_eq!(cses_name(url).as_deref(), Some("cses1640"));
        assert_eq!(cses_name("https://cses.fi/problemset/").as_deref(), None);
    }

    /// Cut from the AtCoder page for abc300_a, as served.
    #[test]
    fn atcoder_samples_title_and_limits_from_the_english_half() {
        let html = "\t\t<span class=\"h2\">\n\t\t\tA - N-choice question\n\t\t\t<a class=\"btn\" href=\"/e\">Editorial</a>\n\t\t</span>\n\
            <p>\n\t\tTime Limit: 2 sec / Memory Limit: 1024 MiB\n\t\t\n</p>\n\
            <span class=\"lang-ja\"><h3>入力例 1</h3><pre>9 9 9\n</pre></span>\n\
            <span class=\"lang-en\">\n<h3>Sample Input 1</h3><pre>3 125 175\n200 300 400\n</pre>\n</section>\n</div>\n\
            <div class=\"part\">\n<section>\n<h3>Sample Output 1</h3><pre>2\n</pre>\n<p>We have <var>125+175 = 300</var>.<br />\n\
            <h3>Sample Input 2</h3><pre>1 1 1\n1\n</pre><h3>Sample Output 2</h3><pre>1\n</pre></span>";
        let page = parse(Judge::AtCoder, "u", html).unwrap();
        assert_eq!(page.title.as_deref(), Some("A - N-choice question"));
        assert_eq!(page.short_name().as_deref(), Some("a"));
        let limits = page.limits.unwrap();
        assert_eq!(
            (limits.time, limits.memory_kb),
            (Duration::from_secs(2), 1024 * 1024)
        );
        assert_eq!(
            page.samples,
            [
                ("3 125 175\n200 300 400\n".to_string(), "2\n".to_string()),
                ("1 1 1\n1\n".to_string(), "1\n".to_string()),
            ]
        );
    }

    #[test]
    fn a_page_without_samples_says_so() {
        let err = parse(Judge::AtCoder, "https://atcoder.jp/x", "<p>Sign in</p>").unwrap_err();
        assert!(err.contains("found no samples"), "{err}");
    }
}
