//! `cpl build` is deterministic: building the same program twice gives the
//! same C++, byte for byte, over the corpus and the conformance cases. A
//! submission that changes between builds can't be reproduced or diffed,
//! and a hash-ordered walk in codegen shows up here first. Enforces
//! performance/R18 in handbook/guidelines/performance-checklist.md.

use std::path::{Path, PathBuf};

/// The `.cp` files one directory down from the corpus and conformance roots.
fn sources() -> Vec<PathBuf> {
    cpl_corpus::programs::corpus_and_conformance(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."),
    )
}

fn build(src: &Path, out: &Path) -> Option<Vec<u8>> {
    let run = cpl_cli::run(&["build", src.to_str().unwrap(), "-o", out.to_str().unwrap()]);
    let cpp = std::fs::read(out).ok();
    let _ = std::fs::remove_file(out);
    (run.code == 0).then_some(cpp).flatten()
}

/// The test is split into `SHARDS` tests by each source's index, so CI
/// spreads the corpus over its shards.
const SHARDS: usize = 8;

fn shard_of(sources: &[PathBuf], shard: usize) -> Vec<&PathBuf> {
    sources.iter().skip(shard).step_by(SHARDS).collect()
}

fn builds_the_same_twice(shard: usize) {
    let out = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("deterministic_cpp_{shard}"));
    std::fs::create_dir_all(&out).unwrap();
    let sources = sources();
    let mine = shard_of(&sources, shard);
    let chunk = mine.len().div_ceil(8);
    let built = std::sync::atomic::AtomicUsize::new(0);
    let differ: Vec<String> = std::thread::scope(|s| {
        let jobs: Vec<_> = mine
            .chunks(chunk)
            .enumerate()
            .map(|(n, part)| {
                let (out, built) = (out.join(format!("{n}.cpp")), &built);
                s.spawn(move || {
                    let mut differ = Vec::new();
                    for src in part {
                        let Some(first) = build(src, &out) else {
                            continue;
                        };
                        built.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        if build(src, &out).as_ref() != Some(&first) {
                            differ.push(src.display().to_string());
                        }
                    }
                    differ
                })
            })
            .collect();
        jobs.into_iter().flat_map(|j| j.join().unwrap()).collect()
    });
    let built = built.into_inner();
    assert!(
        built >= 1000 / SHARDS,
        "only {built} corpus and conformance programs built in shard {shard}"
    );
    assert!(
        differ.is_empty(),
        "{} programs build to different C++ each time:\n{}",
        differ.len(),
        differ.join("\n")
    );
}

macro_rules! shards {
    ($($test:ident = $shard:literal),* $(,)?) => {
        $(
            #[test]
            fn $test() {
                builds_the_same_twice($shard);
            }
        )*
        #[test]
        fn building_a_program_twice_shards_cover_every_source_once() {
            let sources = sources();
            let mut covered: Vec<&PathBuf> =
                [$($shard),*].into_iter().flat_map(|k| shard_of(&sources, k)).collect();
            covered.sort();
            let mut all: Vec<&PathBuf> = sources.iter().collect();
            all.sort();
            assert_eq!(covered, all);
            assert_eq!([$($shard),*].len(), SHARDS);
        }
    };
}

shards! {
    building_a_program_twice_gives_the_same_cpp_0 = 0,
    building_a_program_twice_gives_the_same_cpp_1 = 1,
    building_a_program_twice_gives_the_same_cpp_2 = 2,
    building_a_program_twice_gives_the_same_cpp_3 = 3,
    building_a_program_twice_gives_the_same_cpp_4 = 4,
    building_a_program_twice_gives_the_same_cpp_5 = 5,
    building_a_program_twice_gives_the_same_cpp_6 = 6,
    building_a_program_twice_gives_the_same_cpp_7 = 7,
}
