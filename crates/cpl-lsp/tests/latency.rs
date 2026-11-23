//! Latency budgets for the shared front end, measured end to end.
//!
//! The budgets are the P0 targets: a front-end check in under 50 ms, and
//! diagnostics and hover from the language server in under 100 ms. They are
//! asserted on the median of many warm samples, and the 90th percentile must
//! stay within twice the budget, so a few samples descheduled by a loaded
//! machine (the gate runs every test at once) don't fail it but a slow path
//! hit on most keystrokes does. They run on files larger than any contest
//! solution (a 500-line file is the plan's reference size; the 5,000-line
//! file is 10x that).
//!
//! The budgets describe the tool people run, which is an optimized build.
//! `cargo test --release` asserts them at both sizes. A plain `cargo test`
//! is an unoptimized build, several times slower: it asserts the budgets
//! at the 500-line reference size, and prints the 5,000-line numbers
//! without asserting them (a cold check of 5,000 lines takes about 55 ms
//! unoptimized, against 18 ms optimized). To see the optimized numbers:
//!
//! ```text
//! cargo test --release -p cpl-lsp --test latency -- --nocapture
//! ```

use cpl_check::Database;
use cpl_json::{obj, Value};
use cpl_lsp::Server;
use std::time::{Duration, Instant};

/// The allocator `cpl lsp` runs with (crates/cpl-cli/src/main.rs), so the
/// numbers are the editor's.
#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

const CHECK_BUDGET: Duration = Duration::from_millis(50);
/// The cold check of the 5,000-line file on Windows. Hosted Windows runners
/// take 2-3x macOS's time on the same commit (61-81 ms median against
/// 24-39 ms, every run since the step was added), so this is the platform's
/// gap, not a regression; `plan/domains/tools/metrics.toml` tracks it
/// against the 50 ms target. Every other size and OS keeps `CHECK_BUDGET`.
const WINDOWS_LARGE_CHECK_BUDGET: Duration = Duration::from_millis(100);
const LSP_BUDGET: Duration = Duration::from_millis(100);
const URI: &str = "file:///bench/solution.cp";
/// The plan's reference file size. Budgets hold here in every profile.
const REFERENCE_LINES: usize = 500;

/// A program of about `lines` lines shaped like a contest solution: typed
/// input, domains, a struct, and many small fns that call each other and
/// index arrays by domain.
fn program(lines: usize) -> String {
    let mut src = String::from(
        "let n: 1..=2e5 = read()\nlet m: 1..=2e5 = read()\ndomain V = 1..=n\ndomain E = 0..m\n\n\
         struct Road:\n    u, v: V\n    len: 1..=1e9\n    cost: 1..=1e9\n\n\
         let roads: [E]Road = read()\nvar dist: [V]?Int = None\nvar deg: [V]Int = 0\nvar total = 0\n\n",
    );
    let mut i = 0usize;
    while src.lines().count() + 12 < lines {
        let prev = if i == 0 {
            "0".to_string()
        } else {
            format!("step_{}(r, acc)", i - 1)
        };
        src += &format!(
            "fn step_{i}(r: Road, acc: Int) -> Int:\n    var best = acc\n    for e in E:\n        let other = roads[e]\n        if other.len > r.len and deg[other.u] > {i}:\n            best max= other.cost + (dist[other.v] ?? 0)\n        deg[r.v] += 1\n    let carry = {prev}\n    \"step {i}: {{best}} {{carry}}\"\n    best + carry\n\n",
        );
        i += 1;
    }
    src += &format!(
        "for e in E:\n    let r = roads[e]\n    total += step_{}(r, total)\nprint(total)\n",
        i.saturating_sub(1)
    );
    src
}

/// Where a keystroke lands: inside the body of a fn in the middle.
fn edit_point(src: &str) -> usize {
    let middle = src.len() / 2;
    middle
        + src[middle..]
            .find("best max= ")
            .expect("a fn body after the middle")
        + "best max= ".len()
}

struct Samples(Vec<Duration>);

impl Samples {
    fn median(&mut self) -> Duration {
        self.0.sort();
        self.0[self.0.len() / 2]
    }

    fn max(&self) -> Duration {
        *self.0.iter().max().unwrap()
    }

    fn p90(&mut self) -> Duration {
        self.0.sort();
        self.0[(self.0.len() - 1) * 9 / 10]
    }
}

fn ms(d: Duration) -> String {
    format!("{:.3} ms", d.as_secs_f64() * 1e3)
}

fn report(name: &str, lines: usize, samples: &mut Samples, budget: Duration) {
    let (median, p90, max) = (samples.median(), samples.p90(), samples.max());
    let enforced = !cfg!(debug_assertions) || lines <= REFERENCE_LINES;
    let note = if enforced {
        ""
    } else {
        "   (unoptimized build: not asserted)"
    };
    println!(
        "{name:<44} {lines:>5} lines   median {:>10}   p90 {:>10}   max {:>10}   budget {}{note}",
        ms(median),
        ms(p90),
        ms(max),
        ms(budget)
    );
    assert!(
        !enforced || median < budget,
        "{name} on {lines} lines took {} at the median (budget {})",
        ms(median),
        ms(budget)
    );
    assert!(
        !enforced || p90 < 2 * budget,
        "{name} on {lines} lines took {} at the 90th percentile (budget {} at the median)",
        ms(p90),
        ms(budget)
    );
}

/// A row of the table with no budget: a cost the editor pays off the
/// path to the first answer.
fn inform(name: &str, lines: usize, samples: &mut Samples) {
    let (median, p90, max) = (samples.median(), samples.p90(), samples.max());
    println!(
        "{name:<44} {lines:>5} lines   median {:>10}   p90 {:>10}   max {:>10}   budget none",
        ms(median),
        ms(p90),
        ms(max)
    );
}

/// The text after typing `typed` characters of a new expression at `at`.
fn typed(src: &str, at: usize, typed: usize) -> String {
    let insert = "other.len * 2 + deg[other.v] + ";
    format!(
        "{}{}{}",
        &src[..at],
        &insert[..typed % (insert.len() + 1)],
        &src[at..]
    )
}

fn check_latency(lines: usize) {
    let src = program(lines);
    let actual = src.lines().count();
    let at = edit_point(&src);

    // Cold: a fresh database, as `cpl check` and `cpl run` see the file.
    let mut cold = Samples(Vec::new());
    for _ in 0..10 {
        let mut db = Database::new();
        db.set_file("a.cp", src.as_str());
        let (analysis, took) = time_cpu(|| db.analyze("a.cp").unwrap());
        cold.0.push(took);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:#?}",
            &analysis.diagnostics[..1]
        );
    }
    let cold_budget = if cfg!(windows) && lines > REFERENCE_LINES {
        WINDOWS_LARGE_CHECK_BUDGET
    } else {
        CHECK_BUDGET
    };
    report("check, cold", actual, &mut cold, cold_budget);

    // Warm: one keystroke at a time in a fn body, as the editor sees it.
    let mut db = Database::new();
    db.set_file("a.cp", src.as_str());
    db.analyze("a.cp");
    let mut warm = Samples(Vec::new());
    for key in 0..200 {
        let text = typed(&src, at, key);
        let (_, took) = time_cpu(|| {
            db.set_file("a.cp", text.as_str());
            db.analyze("a.cp").unwrap()
        });
        warm.0.push(took);
    }
    report("check, after a keystroke", actual, &mut warm, CHECK_BUDGET);

    // The keystrokes re-checked only the fn being typed in.
    let before = db.stats();
    let unseen = format!("{}deg[other.u] - {}", &src[..at], &src[at..]);
    db.set_file("a.cp", unseen.as_str());
    db.analyze("a.cp");
    let after = db.stats();
    assert_eq!(after.items_parsed - before.items_parsed, 1);
    assert_eq!(after.items_checked - before.items_checked, 1);
}

fn message(id: Option<u32>, method: &str, params: Value) -> Value {
    let mut members = vec![("jsonrpc".to_string(), Value::from("2.0"))];
    if let Some(id) = id {
        members.push(("id".to_string(), id.into()));
    }
    members.push(("method".to_string(), method.into()));
    members.push(("params".to_string(), params));
    Value::Object(members.into_iter().collect())
}

fn position(src: &str, offset: usize) -> Value {
    let line = src[..offset].matches('\n').count() as u32;
    let col = (offset - src[..offset].rfind('\n').map_or(0, |i| i + 1)) as u32;
    obj([("line", line.into()), ("character", col.into())])
}

/// Sends a message as text, as it arrives on the wire, and returns the
/// replies sent at once as text, as they leave, with their time; then the
/// time until the deferred replies (the lints and proofs) have left too.
/// Parsing and printing are part of the cost. Timed by thread CPU time
/// where the platform has it, else wall time.
fn round_trip(server: &mut Server, message: &Value) -> (Vec<String>, Duration, Duration) {
    let wire = message.to_string();
    let (replies, first) = time_cpu(|| {
        let parsed = cpl_json::parse(&wire).unwrap();
        let replies: Vec<String> = server
            .respond(&parsed)
            .iter()
            .map(Value::to_string)
            .collect();
        replies
    });
    let (later, rest) = time_cpu(|| server.idle().iter().map(Value::to_string).count());
    assert!(later <= 1, "one deferred publish at most");
    (replies, first, first + rest)
}

fn lsp_latency(lines: usize) {
    let src = program(lines);
    let actual = src.lines().count();
    let at = edit_point(&src);
    let doc = || obj([("uri", URI.into())]);

    let mut server = Server::new();
    server.handle(&message(Some(1), "initialize", obj([])));
    let open = |uri: &str| {
        message(
            None,
            "textDocument/didOpen",
            obj([(
                "textDocument",
                obj([
                    ("uri", uri.into()),
                    ("version", 1.into()),
                    ("text", src.as_str().into()),
                ]),
            )]),
        )
    };
    // Each copy is a new document, so each open checks the whole file.
    let (mut opened, mut settled) = (Samples(Vec::new()), Samples(Vec::new()));
    for copy in 0..9 {
        let uri = format!("file:///bench/copy_{copy}.cp");
        let (replies, took, all) = round_trip(&mut server, &open(&uri));
        // The checker's diagnostics: any error would mean the program is wrong.
        assert!(
            replies[0].contains(r#""diagnostics":["#) && !replies[0].contains(r#""severity":1"#),
            "{}",
            &replies[0][..200.min(replies[0].len())]
        );
        opened.0.push(took);
        settled.0.push(all);
    }
    report(
        "lsp: open file, first diagnostics",
        actual,
        &mut opened,
        LSP_BUDGET,
    );
    inform("lsp: open file, lints and proofs too", actual, &mut settled);
    round_trip(&mut server, &open(URI));

    // Type an expression one character at a time, with a hover in between.
    let insert = "other.len * 2 + deg[other.v] + ";
    let mut change = Samples(Vec::new());
    let mut hover = Samples(Vec::new());
    let mut complete = Samples(Vec::new());
    let mut signature = Samples(Vec::new());
    let mut hints = Samples(Vec::new());
    let mut refs = Samples(Vec::new());
    let mut rename = Samples(Vec::new());
    let mut text = src.clone();
    for (i, ch) in insert.char_indices() {
        let cursor = position(&text, at + i);
        let edit = message(
            None,
            "textDocument/didChange",
            obj([
                ("textDocument", doc()),
                (
                    "contentChanges",
                    vec![obj([
                        ("range", obj([("start", cursor.clone()), ("end", cursor)])),
                        ("text", ch.to_string().into()),
                    ])]
                    .into(),
                ),
            ]),
        );
        text.insert(at + i, ch);
        let (replies, took, _) = round_trip(&mut server, &edit);
        assert_eq!(replies.len(), 1);
        assert!(replies[0].contains("publishDiagnostics"));
        change.0.push(took);

        // Hover `roads` on the line above the edit, in the half-typed fn.
        let target = text[..at].rfind("roads[e]").unwrap();
        let request = message(
            Some(100 + i as u32),
            "textDocument/hover",
            obj([
                ("textDocument", doc()),
                ("position", position(&text, target + 2)),
            ]),
        );
        let (replies, took, _) = round_trip(&mut server, &request);
        assert!(replies[0].contains("roads: [E]Road"), "{}", replies[0]);
        assert!(
            replies[0].contains("storage: 16 bytes per element, 3.2 MB max"),
            "{}",
            replies[0]
        );
        hover.0.push(took);

        // Complete and show the signature at the cursor, as the editor
        // asks after each character; `other.` must list `Road`'s fields.
        let after = at + i + ch.len_utf8();
        let at_cursor = |id: u32, method: &str| {
            message(
                Some(id),
                method,
                obj([
                    ("textDocument", doc()),
                    ("position", position(&text, after)),
                ]),
            )
        };
        let (replies, took, _) = round_trip(
            &mut server,
            &at_cursor(1000 + i as u32, "textDocument/completion"),
        );
        if text[..after].ends_with("other.") {
            assert!(replies[0].contains(r#""label":"len""#), "{}", replies[0]);
        }
        complete.0.push(took);
        let (_, took, _) = round_trip(
            &mut server,
            &at_cursor(2000 + i as u32, "textDocument/signatureHelp"),
        );
        signature.0.push(took);
        let request = message(
            Some(3000 + i as u32),
            "textDocument/inlayHint",
            obj([("textDocument", doc())]),
        );
        let (_, took, _) = round_trip(&mut server, &request);
        hints.0.push(took);

        // Find the uses of `roads` and rename it, from the half-typed fn.
        let on_roads = |id: u32, method: &str, extra: Vec<(String, Value)>| {
            let mut params = vec![
                ("textDocument".to_string(), doc()),
                ("position".to_string(), position(&text, target + 2)),
            ];
            params.extend(extra);
            message(
                Some(id),
                method,
                Value::Object(params.into_iter().collect()),
            )
        };
        let context = obj([("includeDeclaration", true.into())]);
        let (replies, took, _) = round_trip(
            &mut server,
            &on_roads(
                4000 + i as u32,
                "textDocument/references",
                vec![("context".into(), context)],
            ),
        );
        assert!(replies[0].contains(r#""uri""#), "{}", replies[0]);
        refs.0.push(took);
        let (replies, took, _) = round_trip(
            &mut server,
            &on_roads(
                5000 + i as u32,
                "textDocument/rename",
                vec![("newName".into(), "edges".into())],
            ),
        );
        assert!(
            replies[0].contains(r#""newText":"edges""#),
            "{}",
            replies[0]
        );
        rename.0.push(took);
    }
    report(
        "lsp: keystroke to diagnostics",
        actual,
        &mut change,
        LSP_BUDGET,
    );
    report(
        "lsp: hover in the fn being edited",
        actual,
        &mut hover,
        LSP_BUDGET,
    );
    report(
        "lsp: completion at the cursor",
        actual,
        &mut complete,
        LSP_BUDGET,
    );
    report(
        "lsp: signature help at the cursor",
        actual,
        &mut signature,
        LSP_BUDGET,
    );
    report(
        "lsp: inlay hints for the file",
        actual,
        &mut hints,
        LSP_BUDGET,
    );
    report("lsp: references to a name", actual, &mut refs, LSP_BUDGET);
    report("lsp: rename a name", actual, &mut rename, LSP_BUDGET);
}

#[test]
fn front_end_check_is_under_50_ms() {
    check_latency(500);
    check_latency(5000);
}

#[test]
fn lsp_requests_are_under_100_ms() {
    lsp_latency(500);
    lsp_latency(5000);
}

#[test]
fn the_sample_program_is_the_size_it_claims() {
    for lines in [500usize, 5000] {
        let actual = program(lines).lines().count();
        assert!(
            actual.abs_diff(lines) <= lines / 20,
            "asked for {lines} lines, got {actual}"
        );
    }
}

/// Time this thread has spent on a CPU, from the scheduler's nanosecond
/// count. The server answers on the thread that calls it, so this is the
/// handler's own cost; wall time also counts the time a loaded machine (the
/// gate runs every test at once) leaves the thread descheduled. `None` where
/// the count isn't available.
fn on_cpu() -> Option<Duration> {
    let stat = std::fs::read_to_string("/proc/thread-self/schedstat").ok()?;
    let nanos = stat.split_whitespace().next()?.parse().ok()?;
    Some(Duration::from_nanos(nanos))
}

/// Runs `f`, timed by thread CPU time where the platform has it, else wall time.
fn time_cpu<T>(f: impl FnOnce() -> T) -> (T, Duration) {
    let before = on_cpu();
    let start = Instant::now();
    let out = f();
    let elapsed = match (before, on_cpu()) {
        (Some(before), Some(after)) => after - before,
        _ => start.elapsed(),
    };
    (out, elapsed)
}

#[test]
fn the_keystroke_clock_does_not_count_time_off_the_cpu() {
    let Some(before) = on_cpu() else { return };
    std::thread::sleep(Duration::from_millis(200));
    let slept = on_cpu().unwrap() - before;
    assert!(
        slept < Duration::from_millis(20),
        "sleeping 200 ms took {} on the CPU",
        ms(slept)
    );
    let before = on_cpu().unwrap();
    let start = Instant::now();
    let mut x = 0u64;
    while start.elapsed() < Duration::from_millis(50) {
        x = std::hint::black_box(x.wrapping_mul(31).wrapping_add(7));
    }
    assert!(
        on_cpu().unwrap() - before > Duration::from_millis(5),
        "a busy loop took no CPU time"
    );
}

/// The 95th percentile of the keystroke-to-diagnostics round trip while typing
/// `insert` one character at a time at `at`, after opening `src`.
fn keystroke_p95(src: &str, at: usize, insert: &str) -> (Duration, Duration, bool) {
    let mut server = Server::new();
    server.handle(&message(Some(1), "initialize", obj([])));
    let doc = |version: usize, text: &str| {
        obj([
            ("uri", URI.into()),
            ("version", (version as u32).into()),
            ("text", text.into()),
        ])
    };
    server.handle(&message(
        None,
        "textDocument/didOpen",
        obj([("textDocument", doc(1, src))]),
    ));
    let mut samples = Samples(Vec::new());
    let mut errors = false;
    for round in 0..3 {
        for typed in 1..=insert.len() {
            let text = format!("{}{}{}", &src[..at], &insert[..typed], &src[at..]);
            let change = message(
                None,
                "textDocument/didChange",
                obj([
                    ("textDocument", doc(2 + round * insert.len() + typed, "")),
                    (
                        "contentChanges",
                        Value::Array(vec![obj([("text", text.as_str().into())])]),
                    ),
                ]),
            );
            let (replies, took, _) = round_trip(&mut server, &change);
            assert!(
                replies.iter().any(|r| r.contains("publishDiagnostics")),
                "no diagnostics"
            );
            errors |= replies.iter().any(|r| r.contains(r#""severity":1"#));
            samples.0.push(took);
        }
    }
    let median = samples.median();
    samples.0.sort();
    (samples.0[(samples.0.len() - 1) * 95 / 100], median, errors)
}

/// The budget at the 95th percentile: the LSP budget in an optimized build,
/// twice it unoptimized (several times slower, and the gate runs every test at
/// once).
fn p95_budget() -> Duration {
    if cfg!(debug_assertions) {
        2 * LSP_BUDGET
    } else {
        LSP_BUDGET
    }
}

/// The ten largest ports, typed into in the middle of an indented body, and a
/// 300-line file with errors throughout: each keystroke's diagnostics take
/// the server less than the budget at the 95th percentile, in CPU time so a
/// loaded machine doesn't fail it.
#[test]
fn keystrokes_on_the_largest_ports_and_a_broken_file_are_within_budget_at_p95() {
    let ports = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus/problems");
    let mut files = Vec::new();
    for author in std::fs::read_dir(&ports).unwrap().flatten() {
        let Ok(entries) = std::fs::read_dir(author.path()) else {
            continue;
        };
        for file in entries.flatten() {
            let path = file.path();
            if path.extension().is_some_and(|e| e == "cp") {
                let src = std::fs::read_to_string(&path).unwrap();
                files.push((src.lines().count(), path, src));
            }
        }
    }
    files.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
    files.truncate(10);
    assert_eq!(files.len(), 10);

    let broken = {
        let mut lines: Vec<String> = program(300).lines().map(String::from).collect();
        for (i, line) in lines.iter_mut().enumerate() {
            if i % 25 == 12 && line.starts_with("    ") {
                *line = line.replacen(')', "", 1).replacen(" = ", " = = ", 1);
            }
        }
        lines.join("\n") + "\n"
    };
    let budget = p95_budget();
    let mut worst = Duration::ZERO;
    let mut cases: Vec<(String, String)> = files
        .into_iter()
        .map(|(_, path, src)| {
            let parent = path
                .parent()
                .and_then(|p| p.file_name())
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default();
            let file = path.file_name().unwrap().to_string_lossy();
            (format!("{parent}/{file}"), src)
        })
        .collect();
    cases.push(("300-line file with errors".into(), broken));
    for (index, (name, src)) in cases.iter().enumerate() {
        let middle = src.len() / 2;
        let at = middle
            + src[middle..]
                .find("\n    ")
                .expect("an indented line after the middle")
            + 5;
        let (p95, median, errors) = keystroke_p95(src, at, "best = best + 1 + ");
        let lines = src.lines().count();
        println!(
            "lsp: keystroke on {name:<32} {lines:>5} lines   median {:>10}   p95 {:>10}   budget {}",
            ms(median),
            ms(p95),
            ms(budget)
        );
        if index == cases.len() - 1 {
            assert!(errors, "the broken file has no errors");
        }
        worst = worst.max(p95);
        assert!(
            p95 < budget,
            "a keystroke on {name} took {} at the 95th percentile",
            ms(p95)
        );
    }
    println!("lsp: worst keystroke p95 {}", ms(worst));
}

fn p95(samples: &mut [Duration]) -> Duration {
    samples.sort();
    samples[(samples.len() - 1) * 95 / 100]
}

fn median(samples: &mut [Duration]) -> Duration {
    samples.sort();
    samples[samples.len() / 2]
}

fn line_col_of(src: &str, offset: usize) -> (u32, u32) {
    let before = &src[..offset];
    let line = before.bytes().filter(|&b| b == b'\n').count() as u32;
    let col = before
        .rsplit_once('\n')
        .map_or(before.len(), |(_, tail)| tail.len()) as u32;
    (line, col)
}

/// Goal benchmark: diagnostics, hover, and completion p95 are all under
/// `LSP_BUDGET` (100 ms) on the largest corpus file (both by lines and by
/// bytes).
#[test]
fn largest_corpus_file_diagnostics_hover_and_completion_p95_under_100ms() {
    let ports = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus/problems");
    let mut files = Vec::new();
    for problem in std::fs::read_dir(&ports).unwrap().flatten() {
        let Ok(entries) = std::fs::read_dir(problem.path()) else {
            continue;
        };
        for file in entries.flatten() {
            let path = file.path();
            if path.extension().is_some_and(|e| e == "cp") {
                let src = std::fs::read_to_string(&path).unwrap();
                files.push((src.lines().count(), src.len(), path, src));
            }
        }
    }
    assert!(!files.is_empty(), "corpus/problems has no .cp files");

    let by_lines = files
        .iter()
        .max_by(|a, b| {
            a.0.cmp(&b.0)
                .then_with(|| a.1.cmp(&b.1))
                .then_with(|| b.2.cmp(&a.2))
        })
        .unwrap()
        .clone();
    let by_bytes = files
        .iter()
        .max_by(|a, b| {
            a.1.cmp(&b.1)
                .then_with(|| a.0.cmp(&b.0))
                .then_with(|| b.2.cmp(&a.2))
        })
        .unwrap()
        .clone();

    let mut targets = vec![by_lines];
    if by_bytes.2 != targets[0].2 {
        targets.push(by_bytes);
    }

    let budget = p95_budget();
    for (lines, bytes, path, src) in targets {
        let label = format!(
            "{}/{}",
            path.parent()
                .and_then(|p| p.file_name())
                .unwrap_or_default()
                .to_string_lossy(),
            path.file_name().unwrap_or_default().to_string_lossy()
        );
        let uri = format!("file://{}", path.canonicalize().unwrap().display());
        let mut server = Server::new();
        server.handle(&message(Some(1), "initialize", obj([])));
        server.handle(&message(None, "initialized", obj([])));
        server.handle(&message(
            None,
            "textDocument/didOpen",
            obj([(
                "textDocument",
                obj([
                    ("uri", uri.as_str().into()),
                    ("languageId", "cp".into()),
                    ("version", 1.into()),
                    ("text", src.as_str().into()),
                ]),
            )]),
        ));

        // 1. Diagnostics p95 across 40 warm edits in the middle of the file.
        let middle = src.len() / 2;
        let edit_at = middle
            + src[middle..]
                .find("\n    ")
                .expect("an indented line after the middle")
            + 5;
        let mut diag_samples = Vec::with_capacity(40);
        let mut version = 2usize;
        for i in 0..40 {
            let edited = if i % 2 == 0 {
                format!(
                    "{}let _probe = {i}\n    {}",
                    &src[..edit_at],
                    &src[edit_at..]
                )
            } else {
                src.clone()
            };
            let req = message(
                None,
                "textDocument/didChange",
                obj([
                    (
                        "textDocument",
                        obj([("uri", uri.as_str().into()), ("version", version.into())]),
                    ),
                    (
                        "contentChanges",
                        Value::Array(vec![obj([("text", edited.as_str().into())])]),
                    ),
                ]),
            );
            version += 1;
            let (replies, took, _) = round_trip(&mut server, &req);
            assert!(
                replies.iter().any(|r| r.contains("publishDiagnostics")),
                "no diagnostics on {label}"
            );
            diag_samples.push(took);
        }
        // Restore original text before hover/completion.
        server.handle(&message(
            None,
            "textDocument/didChange",
            obj([
                (
                    "textDocument",
                    obj([("uri", uri.as_str().into()), ("version", version.into())]),
                ),
                (
                    "contentChanges",
                    Value::Array(vec![obj([("text", src.as_str().into())])]),
                ),
            ]),
        ));

        // 2. Hover p95 across 40 identifier positions spread through the file.
        let mut hover_positions = Vec::new();
        for (line_idx, line) in src.lines().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            if let Some(col) = line.find(|c: char| c.is_ascii_alphabetic()) {
                hover_positions.push((line_idx as u32, col as u32));
            }
        }
        assert!(!hover_positions.is_empty());
        let mut hover_samples = Vec::with_capacity(40);
        for i in 0..40 {
            let (line, col) = hover_positions[i * hover_positions.len() / 40];
            let req = message(
                Some(100 + i as u32),
                "textDocument/hover",
                obj([
                    ("textDocument", obj([("uri", uri.as_str().into())])),
                    (
                        "position",
                        obj([("line", line.into()), ("character", col.into())]),
                    ),
                ]),
            );
            let (replies, took, _) = round_trip(&mut server, &req);
            assert_eq!(replies.len(), 1);
            hover_samples.push(took);
        }

        // 3. Completion p95 across 40 requests (alternating bare and dot positions).
        let (edit_line, edit_col) = line_col_of(&src, edit_at);
        let mut dot_positions: Vec<(u32, u32)> = src
            .match_indices('.')
            .filter_map(|(idx, _)| {
                let next = src.as_bytes().get(idx + 1).copied()?;
                next.is_ascii_alphabetic()
                    .then(|| line_col_of(&src, idx + 1))
            })
            .collect();
        if dot_positions.is_empty() {
            dot_positions.push((edit_line, edit_col));
        }
        let mut comp_samples = Vec::with_capacity(40);
        for i in 0..40 {
            let (line, col) = if i % 2 == 0 {
                (edit_line, edit_col)
            } else {
                dot_positions[i % dot_positions.len()]
            };
            let req = message(
                Some(200 + i as u32),
                "textDocument/completion",
                obj([
                    ("textDocument", obj([("uri", uri.as_str().into())])),
                    (
                        "position",
                        obj([("line", line.into()), ("character", col.into())]),
                    ),
                ]),
            );
            let (replies, took, _) = round_trip(&mut server, &req);
            assert_eq!(replies.len(), 1);
            comp_samples.push(took);
        }

        let (d_med, d_p95) = (median(&mut diag_samples), p95(&mut diag_samples));
        let (h_med, h_p95) = (median(&mut hover_samples), p95(&mut hover_samples));
        let (c_med, c_p95) = (median(&mut comp_samples), p95(&mut comp_samples));
        println!(
            "lsp corpus {label} ({lines} lines, {bytes} bytes): \
             diagnostics med={} p95={} | hover med={} p95={} | completion med={} p95={} (budget {})",
            ms(d_med),
            ms(d_p95),
            ms(h_med),
            ms(h_p95),
            ms(c_med),
            ms(c_p95),
            ms(budget),
        );
        assert!(
            d_p95 < budget,
            "{label} diagnostics p95 {} >= {}",
            ms(d_p95),
            ms(budget)
        );
        assert!(
            h_p95 < budget,
            "{label} hover p95 {} >= {}",
            ms(h_p95),
            ms(budget)
        );
        assert!(
            c_p95 < budget,
            "{label} completion p95 {} >= {}",
            ms(c_p95),
            ms(budget)
        );
    }
}
