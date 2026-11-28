//! A run's stack is its judge's limit (`Limits::stack`, spec 9.6) on every backend: recursion
//! past it traps R0601 on the reference interpreter, the engine and the C++, never crashing
//! the host, and a stack the machine will not give is an error saying so.

#[path = "../../cpl-cli/tests/support/cxx.rs"]
mod cxx;

use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

const DEEP: &str = "fn d(n: Int) -> Int = if n == 0: 0 else: (d(n - 1) * 31 + n) % 1000000007\nlet n: Int = read()\nprint(d(n))\n";
/// A judge small enough that the recursion below passes its stack quickly.
const JUDGE: &str = "small=16";
/// Past the 16 MB on every backend: the run's count runs out near 300,000 calls, the C++'s
/// native frames, smaller than the count, near 600,000.
const PAST: &str = "1000000";
const WITHIN: &str = "100000";

fn case() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("cpl-run-stack-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("deep.cp");
    std::fs::write(&path, DEEP).unwrap();
    path
}

/// `cmd` run on `input`, under `ulimit -v kb` when given, as a CI runner refuses a mapping.
fn run(bin: &str, args: &[&str], input: &str, ulimit_kb: Option<u64>) -> Output {
    let mut cmd = match ulimit_kb {
        Some(kb) => {
            let mut c = Command::new("sh");
            c.args(["-c", &format!("ulimit -v {kb} && exec \"$0\" \"$@\""), bin]);
            c
        }
        None => Command::new(bin),
    };
    let mut child = (cmd.args(args))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    use std::io::Write;
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

fn said(o: &Output) -> String {
    format!(
        "exit {:?}, stderr {}",
        o.status.code(),
        String::from_utf8_lossy(&o.stderr)
    )
}

#[test]
fn recursion_past_the_judges_stack_traps_r0601_on_every_backend() {
    let path = case();
    let file = path.to_str().unwrap();
    let next = env!("CARGO_BIN_EXE_cpl-next");
    let oracle = env!("CARGO_BIN_EXE_cpl-next-oracle");
    let cpp = run(next, &[file, "--cpp", "--judge", JUDGE], "", None);
    assert!(cpp.status.success(), "{}", said(&cpp));
    let flags = ["-std=c++17", "-O2", "-w", "-pthread", "-x", "c++"];
    let (bin, _) = cxx::build(&cpp.stdout, &flags).expect("g++ builds it");
    let bin = bin.to_str().unwrap().to_string();
    let backends: [(&str, &str, Vec<&str>); 3] = [
        ("raw", oracle, vec![file, "--raw", "--judge", JUDGE]),
        ("engine", next, vec![file, "--judge", JUDGE]),
        ("c++", &bin, vec![]),
    ];
    for (name, bin, args) in &backends {
        let o = run(bin, args, WITHIN, None);
        assert!(o.status.success(), "{name} within the stack: {}", said(&o));
        let o = run(bin, args, PAST, None);
        assert!(
            o.status.code() == Some(3) && String::from_utf8_lossy(&o.stderr).contains("R0601"),
            "{name} past the stack: {}",
            said(&o)
        );
    }
}

#[test]
fn a_stack_the_machine_will_not_give_is_an_error_not_a_crash() {
    let path = case();
    let file = path.to_str().unwrap();
    let next = env!("CARGO_BIN_EXE_cpl-next");
    // 8 GB of address space, as on a CI runner: the generic judge's stack fits, and the
    // recursion grows past the thread's stack until the run's count stops it.
    let limit = Some(8 << 20);
    let o = run(next, &[file, "--judge", JUDGE], PAST, limit);
    assert!(
        String::from_utf8_lossy(&o.stderr).contains("R0601"),
        "{}",
        said(&o)
    );
    let o = run(next, &[file], WITHIN, limit);
    assert!(o.status.success(), "{}", said(&o));
    // A 16 GB judge's stack is more than that address space holds.
    let o = run(next, &[file, "--judge", "big=16384"], WITHIN, limit);
    assert!(
        o.status.code() == Some(2)
            && String::from_utf8_lossy(&o.stderr)
                .contains("will not give the run the judge's 16384 MB stack"),
        "{}",
        said(&o)
    );
}
