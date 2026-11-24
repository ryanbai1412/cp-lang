//! What a built program costs before its first line runs: `main` starts no
//! thread for the profile's stack (0.1 ms of a 0.3 ms `print(1)`), and deep
//! recursion still runs on every stack limit a judge may set.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// `source` built with `cpl build` and `g++ -O2`, or `None` without g++.
fn built(test: &str, source: &[u8]) -> Option<PathBuf> {
    Command::new("g++").arg("--version").output().ok()?;
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(test);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("a.cp");
    std::fs::write(&path, source).unwrap();
    let cpl = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .arg("build")
        .arg(&path)
        .output()
        .unwrap();
    assert!(cpl.status.success(), "{cpl:?}");
    let binary = path.with_extension("bin");
    let gxx = Command::new("g++")
        .args(["-std=c++17", "-O2", "-o"])
        .arg(&binary)
        .arg(path.with_extension("cpp"))
        .output()
        .unwrap();
    assert!(gxx.status.success(), "{gxx:?}");
    Some(binary)
}

/// Waiting on its input, the program is one thread: it got its stack
/// without starting another.
#[cfg(target_os = "linux")]
#[test]
fn main_starts_no_thread_for_the_stack() {
    use std::io::Write;
    let Some(binary) = built("startup-threads", b"let n: Int = read()\nprint(n)\n") else {
        return;
    };
    let mut child = Command::new("sh")
        .arg("-c")
        .arg("ulimit -S -s 8192 && exec \"$0\"")
        .arg(&binary)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let tasks = Path::new("/proc").join(child.id().to_string()).join("task");
    let blocked = |t: &Path| {
        let stat = std::fs::read_to_string(t.join("stat")).unwrap_or_default();
        stat.rsplit(')')
            .next()
            .is_some_and(|s| s.trim_start().starts_with('S'))
    };
    let mut seen = 0;
    let give_up = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while std::time::Instant::now() < give_up {
        let comm = std::fs::read_to_string(tasks.with_file_name("comm")).unwrap_or_default();
        let all: Vec<_> = std::fs::read_dir(&tasks)
            .unwrap()
            .map(|e| e.unwrap().path())
            .collect();
        if comm.trim_end() == "a.bin" && all.iter().all(|t| blocked(t)) {
            seen = all.len();
            break;
        }
        std::thread::yield_now();
    }
    child.stdin.take().unwrap().write_all(b"7\n").unwrap();
    let out = child.wait_with_output().unwrap();
    assert_eq!(String::from_utf8_lossy(&out.stdout), "7\n");
    assert_eq!(seen, 1, "threads while the program waits on its input");
}

/// 10^6 calls deep, on the 8 MB a shell gives, on an unlimited stack,
/// and on a limit the program can't raise.
#[test]
fn a_million_calls_deep_run_on_every_stack_limit() {
    let source = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/stack/path_dfs.cp"
    ))
    .unwrap();
    let Some(binary) = built("startup-deep", &source) else {
        return;
    };
    for limit in [
        "ulimit -s 8192",
        "ulimit -s unlimited",
        "ulimit -s 4096 && ulimit -H -s 4096",
    ] {
        let run = Command::new("sh")
            .arg("-c")
            .arg(format!(
                "{{ {limit}; }} 2>/dev/null || exit 77; echo 1000000 | exec \"$0\""
            ))
            .arg(&binary)
            .output()
            .unwrap();
        if run.status.code() == Some(77) {
            continue;
        }
        assert!(run.status.success(), "{limit}: {run:?}");
        assert_eq!(
            String::from_utf8_lossy(&run.stdout),
            "1000000 999999\n",
            "{limit}"
        );
    }
}
