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

const CHECK_BUDGET: Duration = Duration::from_millis(50);
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
         let roads: [E]Road = read()\nvar dist: [V]?Int = none\nvar deg: [V]Int = 0\nvar total = 0\n\n",
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
    report("check, cold", actual, &mut cold, CHECK_BUDGET);

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
/// replies as text, as they leave. Parsing and printing are part of the cost.
/// Timed by thread CPU time where the platform has it, else wall time.
fn round_trip(server: &mut Server, message: &Value) -> (Vec<String>, Duration) {
    let wire = message.to_string();
    time_cpu(|| {
        let parsed = cpl_json::parse(&wire).unwrap();
        server
            .handle(&parsed)
            .iter()
            .map(Value::to_string)
            .collect()
    })
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
    let mut opened = Samples(Vec::new());
    for copy in 0..9 {
        let uri = format!("file:///bench/copy_{copy}.cp");
        let (replies, took) = round_trip(&mut server, &open(&uri));
        // Lints included: only errors would mean the program is wrong.
        assert!(
            replies[0].contains(r#""diagnostics":["#) && !replies[0].contains(r#""severity":1"#),
            "{}",
            &replies[0][..200.min(replies[0].len())]
        );
        opened.0.push(took);
    }
    report(
        "lsp: open file, first diagnostics",
        actual,
        &mut opened,
        LSP_BUDGET,
    );
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
        let (replies, took) = round_trip(&mut server, &edit);
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
        let (replies, took) = round_trip(&mut server, &request);
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
        let (replies, took) = round_trip(
            &mut server,
            &at_cursor(1000 + i as u32, "textDocument/completion"),
        );
        if text[..after].ends_with("other.") {
            assert!(replies[0].contains(r#""label":"len""#), "{}", replies[0]);
        }
        complete.0.push(took);
        let (_, took) = round_trip(
            &mut server,
            &at_cursor(2000 + i as u32, "textDocument/signatureHelp"),
        );
        signature.0.push(took);
        let request = message(
            Some(3000 + i as u32),
            "textDocument/inlayHint",
            obj([("textDocument", doc())]),
        );
        let (_, took) = round_trip(&mut server, &request);
        hints.0.push(took);

        // Find the uses of `roads` and rename it, from the half-typed fn.
        let on_roads = |id: u32, method: &str, extra: Vec<(String, Value)>| {
            let mut params = vec![
                ("textDocument".to_string(), doc()),
                ("position".to_string(), position(&text, target + 2)),
            ];
            params.extend(extra);
            message(Some(id), method, Value::Object(params.into_iter().collect()))
        };
        let context = obj([("includeDeclaration", true.into())]);
        let (replies, took) = round_trip(
            &mut server,
            &on_roads(
                4000 + i as u32,
                "textDocument/references",
                vec![("context".into(), context)],
            ),
        );
        assert!(replies[0].contains(r#""uri""#), "{}", replies[0]);
        refs.0.push(took);
        let (replies, took) = round_trip(
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
    assert!(slept < Duration::from_millis(20), "sleeping 200 ms took {} on the CPU", ms(slept));
    let before = on_cpu().unwrap();
    let start = Instant::now();
    let mut x = 0u64;
    while start.elapsed() < Duration::from_millis(50) {
        x = std::hint::black_box(x.wrapping_mul(31).wrapping_add(7));
    }
    assert!(on_cpu().unwrap() - before > Duration::from_millis(5), "a busy loop took no CPU time");
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
    server.handle(&message(None, "textDocument/didOpen", obj([("textDocument", doc(1, src))])));
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
                    ("contentChanges", Value::Array(vec![obj([("text", text.as_str().into())])])),
                ]),
            );
            let (replies, took) = round_trip(&mut server, &change);
            assert!(replies.iter().any(|r| r.contains("publishDiagnostics")), "no diagnostics");
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
        let Ok(entries) = std::fs::read_dir(author.path()) else { continue };
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
        .map(|(_, path, src)| (path.file_name().unwrap().to_string_lossy().into_owned(), src))
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
        assert!(p95 < budget, "a keystroke on {name} took {} at the 95th percentile", ms(p95));
    }
    println!("lsp: worst keystroke p95 {}", ms(worst));
}
