//! A traced run on the register VM records the same trace as on the
//! tree-walker, over every program in `tests/programs`.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn traced(program: &Path, tree: bool) -> (Vec<u8>, Vec<u8>) {
    let input = program.with_extension("in");
    let stdin = match std::fs::File::open(&input) {
        Ok(file) => Stdio::from(file),
        Err(_) => Stdio::null(),
    };
    let mut cpl = Command::new(env!("CARGO_BIN_EXE_cpl"));
    cpl.args([
        "run",
        program.to_str().unwrap(),
        "--trace",
        "--format",
        "json",
    ])
    .stdin(stdin);
    if tree {
        cpl.env("CPL_TRACE_ENGINE", "tree");
    } else {
        cpl.env_remove("CPL_TRACE_ENGINE");
    }
    let out = cpl.output().unwrap();
    (out.stdout, out.stderr)
}

#[test]
fn the_vm_records_what_the_tree_walker_records() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/programs");
    let mut programs: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|e| e == "cp"))
        .collect();
    programs.sort();
    assert!(programs.len() > 250, "{} programs", programs.len());
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    let chunks: Vec<_> = programs.chunks(programs.len().div_ceil(threads)).collect();
    let differ: Vec<String> = std::thread::scope(|scope| {
        let workers: Vec<_> = chunks
            .iter()
            .map(|chunk| {
                scope.spawn(move || {
                    chunk
                        .iter()
                        .filter(|program| traced(program, true) != traced(program, false))
                        .map(|program| program.file_stem().unwrap().to_string_lossy().into_owned())
                        .collect::<Vec<_>>()
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
        "the VM's trace differs from the tree-walker's: {differ:?}"
    );
}
