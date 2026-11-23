//! The trace viewer rendered by a real browser: each debugging demo's
//! `cpl run --trace` page, loaded in headless Chrome, shows its capture,
//! its top-level variables and no script error; a stress trace opens on its
//! first divergence, `#step=N` opens on step N and a trapped run says where.
//! Skipped, with a note, when no Chrome is found (`CPL_CHROME` names one).

use std::path::{Path, PathBuf};
use std::process::Command;

fn chrome() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("CPL_CHROME") {
        return Some(path.into());
    }
    let names = ["google-chrome", "chromium", "chromium-browser", "chrome"];
    let path = std::env::var_os("PATH")?;
    let found = std::env::split_paths(&path)
        .flat_map(|dir| names.iter().map(move |name| dir.join(name)))
        .find(|candidate| candidate.is_file());
    if found.is_none() {
        assert!(
            std::env::var_os("CI").is_none(),
            "no Chrome on PATH in CI (set CPL_CHROME); the viewer would go untested"
        );
        eprintln!("skipped: no Chrome on PATH (set CPL_CHROME)");
    }
    found
}

/// The rendered DOM of `url` and the browser's log.
fn render(chrome: &Path, url: &str) -> (String, String) {
    let out = Command::new(chrome)
        .args([
            "--headless=new",
            "--disable-gpu",
            "--no-sandbox",
            "--no-first-run",
            "--enable-logging=stderr",
            "--v=0",
            "--virtual-time-budget=3000",
            "--dump-dom",
        ])
        .arg(url)
        .output()
        .expect("chrome runs");
    let text = |b: &[u8]| String::from_utf8_lossy(b).into_owned();
    let log = text(&out.stderr);
    let errors: Vec<&str> = log.lines().filter(|l| l.contains("Uncaught")).collect();
    assert!(
        errors.is_empty(),
        "{url}: script errors:\n{}",
        errors.join("\n")
    );
    (text(&out.stdout), log)
}

fn url(page: &Path, hash: &str) -> String {
    format!("file://{}{hash}", page.display())
}

fn scratch(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("trace_browser_{name}_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn copy_demo(dir: &Path, demo: &str) {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/debugging");
    std::fs::copy(
        source.join(format!("{demo}.cp")),
        dir.join(format!("{demo}.cp")),
    )
    .unwrap();
}

/// `cpl args` in `dir`: its standard error.
fn cpl(dir: &Path, args: &[&str]) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stderr).into_owned()
}

/// `cpl run --trace` of `file` on `input`: the page and its step count.
fn traced(dir: &Path, file: &str, input: &str) -> (PathBuf, usize) {
    std::fs::write(dir.join("case.in"), input).unwrap();
    let err = cpl(dir, &["run", &format!("{file}.cp"), "case.in", "--trace"]);
    let steps = err
        .split_once(".trace.html (")
        .and_then(|(_, rest)| rest.split_once(" steps)"))
        .and_then(|(n, _)| n.parse().ok())
        .unwrap_or_else(|| panic!("no step count in {err}"));
    (dir.join(format!("{file}.trace.html")), steps)
}

/// The text from the element `id` on, tags dropped, spaces collapsed.
fn text_at(dom: &str, id: &str) -> String {
    let at = dom
        .split_once(&format!("id=\"{id}\""))
        .map_or("", |(_, rest)| rest);
    let mut text = String::new();
    let mut in_tag = true;
    for c in at.chars().take(1200) {
        match c {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                text.push(' ');
            }
            _ if !in_tag => text.push(c),
            _ => {}
        }
    }
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[test]
fn every_debugging_demo_renders_in_a_browser() {
    let Some(chrome) = chrome() else { return };
    let demos: [(&str, &str, &[&str]); 3] = [
        (
            "knapsack",
            "3 5\n2 3\n3 4\n4 5\n",
            &["n", "cap", "items", "best"],
        ),
        (
            "shortest_path",
            "3 3\n1 2 1\n2 3 1\n1 3 5\n",
            &["n", "m", "roads", "adj", "dist"],
        ),
        ("shows", "2\n1\n0 1\n3\n0 2\n0 1\n1 1\n", &["n", "shows"]),
    ];
    for (demo, input, globals) in demos {
        let dir = scratch(demo);
        copy_demo(&dir, demo);
        let (page, steps) = traced(&dir, demo, input);
        let (dom, _) = render(&chrome, &url(&page, ""));
        assert!(
            dom.contains(&format!("<h1 id=\"title\">{demo}.cp</h1>")),
            "{demo}: no title"
        );
        let status = text_at(&dom, "status");
        assert!(
            status.starts_with(&format!("Execution captured {steps} steps")),
            "{demo}: the status does not show the {steps}-step capture: {status}"
        );
        for name in ["stdout"].iter().chain(globals) {
            assert!(
                dom.contains(&format!("data-focus=\"var:{name}\"")),
                "{demo}: `{name}` is not in the variable list"
            );
        }
    }
}

#[test]
fn a_stress_trace_opens_on_its_first_divergence_and_a_link_on_its_step() {
    let Some(chrome) = chrome() else { return };
    let dir = scratch("stress");
    copy_demo(&dir, "knapsack");
    let out = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .args(["stress", "knapsack.cp", "--seed", "1", "--trace"])
        .current_dir(&dir)
        .output()
        .unwrap();
    let report =
        String::from_utf8_lossy(&out.stdout).into_owned() + &String::from_utf8_lossy(&out.stderr);
    let page = report
        .lines()
        .find_map(|l| l.trim().strip_prefix("trace "))
        .map(|p| dir.join(p))
        .unwrap_or_else(|| panic!("no trace in {report}"));
    let (dom, _) = render(&chrome, &url(&page, ""));
    let status = text_at(&dom, "status");
    assert!(
        status.starts_with("First divergence at step ")
            && status.contains("stdout[0]: fast 2 ≠ spec 1"),
        "{status}"
    );
    let label = text_at(&dom, "tl-label");
    let total = label
        .split(" / ")
        .nth(1)
        .and_then(|t| t.split(' ').next())
        .unwrap_or_else(|| panic!("no step total in {label}"))
        .to_string();
    let (dom, _) = render(&chrome, &url(&page, "#step=3"));
    let label = text_at(&dom, "tl-label");
    assert!(label.contains(&format!("Step 3 / {total}")), "{label}");
}

#[test]
fn a_trapped_run_says_where_it_trapped() {
    let Some(chrome) = chrome() else { return };
    let dir = scratch("trap");
    let source =
        "let n: 1..=5 = read()\nvar a: [0..n]Int = 0\nfor i in 0..=n:\n    a[i] = i\nprint(a)\n";
    std::fs::write(dir.join("trap.cp"), source).unwrap();
    let (page, steps) = traced(&dir, "trap", "3\n");
    let (dom, _) = render(&chrome, &url(&page, ""));
    let status = text_at(&dom, "status");
    assert!(
        status.starts_with(&format!(
            "Trapped at step {steps}, line 4 index 3 is outside an array dimension of length 3"
        )) && status.contains("Jump to the trap"),
        "{status}"
    );
    let (dom, _) = render(&chrome, &url(&page, &format!("#step={steps}")));
    let locals = text_at(&dom, "locals");
    assert!(locals.starts_with("i 3 "), "{locals}");
}
