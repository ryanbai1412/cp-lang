//! Every command but `fetch`, `self` and `setup` works with no network: the
//! contest flow runs with `curl`, `wget` and `code` replaced by stubs that
//! log and fail, and every proxy pointed at a closed port.
#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, Output};

fn stub(bin: &Path, name: &str, log: &Path) {
    let path = bin.join(name);
    let script = format!(
        "#!/bin/sh\necho \"{name} $*\" >> '{}'\nexit 7\n",
        log.display()
    );
    std::fs::write(&path, script).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
}

#[test]
fn the_contest_flow_makes_no_network_call() {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join("offline");
    let _ = std::fs::remove_dir_all(&root);
    let (bin, dir) = (root.join("bin"), root.join("work"));
    std::fs::create_dir_all(&bin).unwrap();
    std::fs::create_dir_all(&dir).unwrap();
    let log = root.join("network.log");
    for name in ["curl", "wget", "code"] {
        stub(&bin, name, &log);
    }
    let path = format!(
        "{}:{}",
        bin.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let cpl = |args: &[&str]| -> Output {
        let closed = "http://127.0.0.1:9";
        Command::new(env!("CARGO_BIN_EXE_cpl"))
            .args(args)
            .current_dir(&dir)
            .env("PATH", &path)
            .env("HOME", &root)
            .env("CPL_HOME", root.join("cpl-home"))
            .envs(
                [
                    "http_proxy",
                    "https_proxy",
                    "HTTP_PROXY",
                    "HTTPS_PROXY",
                    "ALL_PROXY",
                ]
                .map(|k| (k, closed)),
            )
            .output()
            .expect("cpl runs")
    };
    let new = cpl(&["new", "sum"]);
    assert!(
        new.status.success(),
        "{}",
        String::from_utf8_lossy(&new.stderr)
    );
    std::fs::write(dir.join("sum.cp"), "let n: 1..=10 = read()\nprint(n * 2)\n").unwrap();
    std::fs::write(dir.join("sum.tests/1.in"), "3\n").unwrap();
    std::fs::write(dir.join("sum.tests/1.out"), "6\n").unwrap();
    let flow: &[&[&str]] = &[
        &["check", "sum.cp"],
        &["fix", "sum.cp"],
        &["fmt", "sum.cp"],
        &["lint", "sum.cp"],
        &["run", "sum.cp", "sum.tests/1.in"],
        &["test", "sum.cp"],
        &["explain", "sum.cp"],
        &["build", "sum.cp"],
        &["run", "--release", "sum.cp", "sum.tests/1.in"],
        &["submit", "sum.cp"],
        &["cover", "sum.cp"],
        &["judges"],
        &["--help"],
    ];
    let mut failed = Vec::new();
    for args in flow {
        let out = cpl(args);
        if !out.status.success() {
            failed.push(format!(
                "cpl {}: {}",
                args.join(" "),
                String::from_utf8_lossy(&out.stderr)
            ));
        }
    }
    let calls = std::fs::read_to_string(&log).unwrap_or_default();
    assert!(calls.is_empty(), "network tools called offline:\n{calls}");
    assert!(
        failed.is_empty(),
        "offline failures:\n{}",
        failed.join("\n")
    );
    let submit = cpl(&["submit", "sum.cp"]);
    let report = String::from_utf8_lossy(&submit.stdout);
    assert!(
        report.contains("1 of 1") || report.contains("pass"),
        "{report}"
    );
    // The stubs are what `fetch` reaches, so the log above would show a call.
    let fetch = cpl(&["fetch", "https://codeforces.com/contest/1/problem/A", "f"]);
    assert!(!fetch.status.success());
    let calls = std::fs::read_to_string(&log).unwrap_or_default();
    assert!(
        calls.starts_with("curl "),
        "fetch did not reach the stub: {calls:?}"
    );
    println!(
        "offline: {} commands, 0 network calls; submit: {}",
        flow.len() + 1,
        report.trim()
    );
}
