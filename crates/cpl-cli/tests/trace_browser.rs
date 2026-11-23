//! The trace viewer rendered by a real browser: each debugging demo's
//! `cpl run --trace` page, loaded in headless Chrome, shows its capture,
//! its top-level variables and no script error. Skipped, with a note, when
//! no Chrome is found (`CPL_CHROME` names one).

use std::path::{Path, PathBuf};
use std::process::Command;

fn chrome() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("CPL_CHROME") {
        return Some(path.into());
    }
    let names = ["google-chrome", "chromium", "chromium-browser", "chrome"];
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .flat_map(|dir| names.iter().map(move |name| dir.join(name)))
        .find(|candidate| candidate.is_file())
}

/// The page's rendered DOM and the browser's log.
fn render(chrome: &Path, page: &Path) -> (String, String) {
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
        .arg(format!("file://{}", page.display()))
        .output()
        .expect("chrome runs");
    let text = |b: &[u8]| String::from_utf8_lossy(b).into_owned();
    (text(&out.stdout), text(&out.stderr))
}

fn traced(dir: &Path, demo: &str, input: &str) -> (PathBuf, usize) {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/debugging");
    std::fs::copy(
        source.join(format!("{demo}.cp")),
        dir.join(format!("{demo}.cp")),
    )
    .unwrap();
    std::fs::write(dir.join("case.in"), input).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .args(["run", &format!("{demo}.cp"), "case.in", "--trace"])
        .current_dir(dir)
        .output()
        .unwrap();
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "{err}");
    let steps = err
        .split_once(".trace.html (")
        .and_then(|(_, rest)| rest.split_once(" steps)"))
        .and_then(|(n, _)| n.parse().ok())
        .unwrap_or_else(|| panic!("no step count in {err}"));
    (dir.join(format!("{demo}.trace.html")), steps)
}

#[test]
fn every_debugging_demo_renders_in_a_browser() {
    let Some(chrome) = chrome() else {
        eprintln!("skipped: no Chrome on PATH (set CPL_CHROME)");
        return;
    };
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
        let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("trace_browser_{demo}_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let (page, steps) = traced(&dir, demo, input);
        let (dom, log) = render(&chrome, &page);
        let errors: Vec<&str> = log.lines().filter(|l| l.contains("Uncaught")).collect();
        assert!(
            errors.is_empty(),
            "{demo}: script errors:\n{}",
            errors.join("\n")
        );
        assert!(
            dom.contains(&format!("<h1 id=\"title\">{demo}.cp</h1>")),
            "{demo}: no title"
        );
        let status = dom.split_once("id=\"status\"").map_or("", |(_, rest)| rest);
        let status: String = status.chars().take(400).collect();
        assert!(
            status.contains("Execution captured") && status.contains(&format!("{steps} steps")),
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
