//! Every interactive corpus problem with a judge (`interactor.py`) is
//! solved by its port on the cases its `gen.py`
//! makes, in the interpreter, and as the judge's C++ when a compiler is
//! here; a package with no port yet, by its accepted C++. A port that forgets to flush, reads past
//! the judge's reply or asks too many questions fails here.

#[path = "support/parallel.rs"]
mod parallel;
#[path = "support/shards.rs"]
mod shards;

use parallel::parallel;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::Duration;

/// `gen.py`'s seed and size for the made cases.
const GENERATED: [(u32, u32); 2] = [(1, 8), (2, 64)];

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

fn name_of(dir: &Path) -> String {
    dir.file_name().unwrap().to_string_lossy().to_string()
}

/// Runs `dir`'s port (else its accepted C++) against its judge on every
/// made case: how many runs, and what failed.
fn judge_problem(dir: &Path, cpp: bool) -> (usize, Vec<String>) {
    let name = name_of(dir);
    let reference = match dir.join("port.cp").exists() {
        true => None,
        false => match cpl_contest::package::reference_exe(dir) {
            Ok(exe) => Some(exe),
            Err(e) => return (0, vec![format!("{name}: {e}")]),
        },
    };
    let (mut runs, mut failures) = (0, Vec::new());
    let mut cases: Vec<PathBuf> = Vec::new();
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
    let modes: &[&[&str]] = match (&reference, cpp) {
        (Some(_), _) => &[&[]],
        (None, true) => &[&["run"], &["run", "--release"]],
        (None, false) => &[&["run"]],
    };
    for case in &cases {
        for mode in modes {
            runs += 1;
            let mut judge = Command::new("python3");
            judge.arg("interactor.py").arg(case).arg("--");
            match &reference {
                Some(exe) => judge.arg(exe),
                None => judge
                    .arg(env!("CARGO_BIN_EXE_cpl"))
                    .args(*mode)
                    .arg("port.cp"),
            };
            judge.current_dir(dir);
            let out = within(judge, Duration::from_secs(120));
            if !out.status.success() {
                failures.push(format!(
                    "{name} {} ({}): {}",
                    case.file_name().unwrap().to_string_lossy(),
                    match &reference {
                        Some(_) => "accepted C++".to_string(),
                        None => format!("cpl {} port.cp", mode.join(" ")),
                    },
                    String::from_utf8_lossy(&out.stderr).trim()
                ));
            }
        }
    }
    (runs, failures)
}

/// The interactive problems, sorted.
fn interactive_problems() -> Vec<PathBuf> {
    let mut dirs: Vec<_> = std::fs::read_dir(problems())
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|dir| interactive(dir))
        .collect();
    dirs.sort();
    dirs
}

/// The interactive problems with a judge, each named by its id.
fn judged() -> Vec<PathBuf> {
    (interactive_problems().into_iter())
        .filter(|dir| dir.join("interactor.py").exists())
        .collect()
}

fn keys() -> Vec<String> {
    judged().iter().map(|dir| name_of(dir)).collect()
}

/// The judged problems are split into `SHARDS` tests by a hash of their id.
const SHARDS: usize = 4;

#[test]
fn every_interactive_problem_has_a_judge() {
    let unjudged: Vec<String> = (interactive_problems().iter())
        .filter(|dir| !dir.join("interactor.py").exists())
        .map(|dir| name_of(dir))
        .collect();
    assert!(
        unjudged.is_empty(),
        "interactive problems need an interactor.py to be judged: {unjudged:?}"
    );
}

fn every_interactive_port_satisfies_its_judge(k: usize) {
    if !has("python3") {
        eprintln!("interactive corpus: skipped, no python3");
        return;
    }
    let cpp = has("g++");
    let judges = shards::mine(judged(), |dir| name_of(dir), k, SHARDS);
    let portless = judges
        .iter()
        .filter(|dir| !dir.join("port.cp").exists())
        .count();
    // Each problem's runs are independent and wait on python and the judge
    // more than on a core, so the problems run side by side.
    let results = parallel(&judges, |dir| judge_problem(dir, cpp));
    let runs: usize = results.iter().map(|(runs, _)| runs).sum();
    let mut failures: Vec<String> = results.into_iter().flat_map(|(_, f)| f).collect();
    failures.sort();
    eprintln!(
        "interactive corpus, shard {k}: {} problems judged, {runs} runs, {} failed; {portless} judged by their accepted C++",
        judges.len(),
        failures.len(),
    );
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

shards::shard_tests!(
    every_interactive_port_satisfies_its_judge,
    keys(),
    the_judge_shards_cover_every_judged_problem,
    [
        every_interactive_port_satisfies_its_judge_0 = 0,
        every_interactive_port_satisfies_its_judge_1 = 1,
        every_interactive_port_satisfies_its_judge_2 = 2,
        every_interactive_port_satisfies_its_judge_3 = 3,
    ]
);

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
