//! A traced run of every program in `tests/programs` records the trace
//! saved next to it in `<name>.trace`. After a change that is meant to
//! change traces, run with `CPL_BLESS=1` and review the diff.
//!
//! A saved trace is the run's variables and steps, one step a line, and
//! what it printed to standard error. Past 8 KB it keeps the first lines
//! and a hash of the rest, so a large run is still checked whole.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const KEPT: usize = 8 << 10;

/// The trace `program` records, as saved.
fn traced(program: &Path) -> String {
    let input = program.with_extension("in");
    let stdin = match std::fs::File::open(&input) {
        Ok(file) => Stdio::from(file),
        Err(_) => Stdio::null(),
    };
    let out = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .args([
            "run",
            program.to_str().unwrap(),
            "--trace",
            "--format",
            "json",
        ])
        .stdin(stdin)
        .output()
        .unwrap();
    let document = String::from_utf8_lossy(&out.stdout).replace("\r\n", "\n");
    // The title and source come first and depend on the checkout.
    let from = document.find("\"variables\":").unwrap_or(0);
    let steps = document[from..]
        .trim_end()
        .replace(",{\"id\":", ",\n{\"id\":");
    let name = program.file_name().unwrap().to_string_lossy();
    let stderr = String::from_utf8_lossy(&out.stderr)
        .replace("\r\n", "\n")
        .replace(program.to_str().unwrap(), &name);
    let mut saved = String::new();
    if steps.len() <= KEPT {
        saved.push_str(&steps);
        saved.push('\n');
    } else {
        let mut cut = steps[..KEPT].rfind('\n').unwrap_or(0);
        while !steps.is_char_boundary(cut) {
            cut -= 1;
        }
        let rest = &steps[cut..];
        saved.push_str(&steps[..cut]);
        saved.push_str(&format!(
            "\n... {} more lines, {} bytes, fnv64 {:016x}\n",
            rest.matches('\n').count(),
            rest.len(),
            fnv64(rest.as_bytes())
        ));
    }
    saved.push_str("stderr:\n");
    saved.push_str(&stderr);
    saved
}

fn fnv64(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, &b| {
        (hash ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

#[test]
fn every_program_records_its_saved_trace() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/programs");
    let mut programs: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|e| e == "cp"))
        .collect();
    programs.sort();
    assert!(programs.len() > 150, "{} programs", programs.len());
    let bless = std::env::var_os("CPL_BLESS").is_some();
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    let chunks: Vec<_> = programs.chunks(programs.len().div_ceil(threads)).collect();
    let differ: Vec<String> = std::thread::scope(|scope| {
        let workers: Vec<_> = chunks
            .iter()
            .map(|chunk| {
                scope.spawn(move || {
                    let mut differ = Vec::new();
                    for program in *chunk {
                        let got = traced(program);
                        let golden = program.with_extension("trace");
                        if bless {
                            std::fs::write(&golden, &got).unwrap();
                            continue;
                        }
                        let want = std::fs::read_to_string(&golden)
                            .unwrap_or_default()
                            .replace("\r\n", "\n");
                        if got != want {
                            let at = got
                                .lines()
                                .zip(want.lines())
                                .position(|(g, w)| g != w)
                                .unwrap_or(got.lines().count().min(want.lines().count()));
                            let line = |text: &str| {
                                text.lines()
                                    .nth(at)
                                    .unwrap_or("")
                                    .chars()
                                    .take(300)
                                    .collect::<String>()
                            };
                            differ.push(format!(
                                "{} line {}:\n  got  {}\n  want {}",
                                golden.file_name().unwrap().to_string_lossy(),
                                at + 1,
                                line(&got),
                                line(&want)
                            ));
                        }
                    }
                    differ
                })
            })
            .collect();
        workers
            .into_iter()
            .flat_map(|worker| worker.join().unwrap())
            .collect()
    });
    assert!(
        differ.is_empty(),
        "{} traces differ from the saved ones (CPL_BLESS=1 records them):\n{}",
        differ.len(),
        differ.join("\n")
    );
}
