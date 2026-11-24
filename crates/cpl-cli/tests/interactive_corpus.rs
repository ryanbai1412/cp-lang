//! Every interactive corpus problem with a judge (`interactor.py`) is
//! solved by its port on each `tests/*.case` and on cases its `gen.py`
//! makes: in the interpreter, and as the judge's C++ when a compiler is
//! here. A port that forgets to flush, reads past
//! the judge's reply or asks too many questions fails here.

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::Duration;

/// Interactive problems with no `interactor.py`, and why.
const KNOWN_NO_JUDGE: &[(&str, &str)] = &[
    ("ioi13p4", "an ideal sketch, not run"),
    ("ioi18p1", "a grader task; spec.cp plays the sample grader"),
];

/// `gen.py`'s seed and size for the made cases.
const GENERATED: [(u32, u32); 2] = [(1, 8), (2, 64)];

/// Ports that don't speak their judge's protocol, and why; their
/// `spec.cp` is run instead. Shrink-only: a port that starts passing must
/// leave the list.
const KNOWN_OTHER_PROTOCOL: &[(&str, &str)] = &[(
    "egoi22p3",
    "port.cp prints YES/NO and the paths for an offline checker",
)];

fn problems() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus/problems")
}

fn interactive(dir: &Path) -> bool {
    let meta = std::fs::read_to_string(dir.join("meta.json")).unwrap_or_default();
    let port = std::fs::read_to_string(dir.join("port.cp")).unwrap_or_default();
    dir.join("interactor.py").exists()
        || meta.contains("\"interactive\"")
        || port.contains("\n@interactor")
}

/// Runs `command`, killing it after `limit`: a run that waits forever on
/// the other side fails instead of hanging the test.
fn within(mut command: Command, limit: Duration) -> Output {
    let child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("runs");
    let pid = child.id();
    let (send, recv) = std::sync::mpsc::channel();
    std::thread::spawn(move || send.send(child.wait_with_output()));
    match recv.recv_timeout(limit) {
        Ok(out) => out.expect("runs"),
        Err(_) => {
            let _ = Command::new("kill").arg(pid.to_string()).status();
            panic!("{command:?} took over {limit:?}");
        }
    }
}

fn has(program: &str) -> bool {
    Command::new(program).arg("--version").output().is_ok()
}

#[test]
fn every_interactive_port_satisfies_its_judge() {
    if !has("python3") {
        eprintln!("interactive corpus: skipped, no python3");
        return;
    }
    let cpp = has("g++");
    let mut names: Vec<_> = std::fs::read_dir(problems())
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|dir| interactive(dir))
        .collect();
    names.sort();
    let (mut runs, mut judged, mut failures, mut unjudged) = (0, 0, Vec::new(), Vec::new());
    for dir in &names {
        let name = dir.file_name().unwrap().to_string_lossy().to_string();
        if !dir.join("interactor.py").exists() {
            unjudged.push(name);
            continue;
        }
        judged += 1;
        let mut cases: Vec<PathBuf> = std::fs::read_dir(dir.join("tests"))
            .map(|entries| entries.map(|e| e.unwrap().path()).collect())
            .unwrap_or_default();
        cases.retain(|p| p.extension().is_some_and(|x| x == "case"));
        cases.sort();
        let made = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("interactive")
            .join(&name);
        std::fs::create_dir_all(&made).unwrap();
        for (seed, size) in GENERATED {
            let out = Command::new("python3")
                .args(["gen.py", &seed.to_string(), &size.to_string()])
                .current_dir(dir)
                .output()
                .expect("python3 runs");
            assert!(out.status.success(), "{name}: gen.py {seed} {size} failed");
            let case = made.join(format!("gen-{seed}-{size}.case"));
            std::fs::write(&case, &out.stdout).unwrap();
            cases.push(case);
        }
        let other = KNOWN_OTHER_PROTOCOL.iter().any(|(n, _)| *n == name);
        let file = if other { "spec.cp" } else { "port.cp" };
        if other {
            let mut judge = Command::new("python3");
            judge
                .arg("interactor.py")
                .arg(&cases[0])
                .arg("--")
                .arg(env!("CARGO_BIN_EXE_cpl"))
                .args(["run", "port.cp"])
                .current_dir(dir);
            let out = within(judge, Duration::from_secs(120));
            assert!(
                !out.status.success(),
                "{name}'s port.cp now satisfies its judge: remove it from KNOWN_OTHER_PROTOCOL"
            );
        }
        let modes: &[&[&str]] = if cpp {
            &[&["run"], &["run", "--release"]]
        } else {
            &[&["run"]]
        };
        for case in &cases {
            for mode in modes {
                runs += 1;
                let mut judge = Command::new("python3");
                judge
                    .arg("interactor.py")
                    .arg(case)
                    .arg("--")
                    .arg(env!("CARGO_BIN_EXE_cpl"))
                    .args(*mode)
                    .arg(file)
                    .current_dir(dir);
                let out = within(judge, Duration::from_secs(120));
                if !out.status.success() {
                    failures.push(format!(
                        "{name} {} (cpl {} {file}): {}",
                        case.file_name().unwrap().to_string_lossy(),
                        mode.join(" "),
                        String::from_utf8_lossy(&out.stderr).trim()
                    ));
                }
            }
        }
    }
    let listed: Vec<_> = KNOWN_NO_JUDGE.iter().map(|(n, _)| n.to_string()).collect();
    assert_eq!(
        unjudged, listed,
        "interactive problems without interactor.py must be listed in KNOWN_NO_JUDGE with why"
    );
    eprintln!(
        "interactive corpus: {judged} problems judged, {runs} runs, {} failed; {} without a judge",
        failures.len(),
        unjudged.len()
    );
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// `cpl run --release` hands the build its input as it arrives: it prints
/// its question before the answer exists, as a judge needs.
#[test]
fn a_release_run_answers_before_its_input_ends() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("live-release");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("a.cp"),
        "print(\"? 1\")\nflush()\nlet x: Int = read()\nprint(\"!\", x * 2)\n",
    )
    .unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .args(["run", "--release", "a.cp"])
        .current_dir(&dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let mut lines = BufReader::new(child.stdout.take().unwrap()).lines();
    let (send, recv) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let question = lines.next().and_then(Result::ok);
        let _ = send.send(question.clone());
        let _ = send.send(lines.next().and_then(Result::ok));
    });
    let question = recv.recv_timeout(Duration::from_secs(120));
    if question.is_err() {
        let _ = child.kill();
    }
    assert_eq!(
        question,
        Ok(Some("? 1".to_string())),
        "no question before the input ended"
    );
    writeln!(stdin, "21").unwrap();
    drop(stdin);
    assert_eq!(
        recv.recv_timeout(Duration::from_secs(60)),
        Ok(Some("! 42".to_string()))
    );
    assert!(child.wait().unwrap().success());
}
