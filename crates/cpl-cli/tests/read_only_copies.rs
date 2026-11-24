//! The C++ each corpus program and conformance case builds to makes exactly
//! the deep copies and `std::function`s its header states (`// copies: N`,
//! `// std_functions: N`; none when absent), so a new one turns the one
//! program that grew it red, by name. A heap value that is only read is read
//! in place: the copy shapes in `NEVER` fail anywhere, and `cpl explain`
//! gives each deep copy its reason.
//! Enforces performance/R12, R13 and R14 in
//! handbook/guidelines/performance-checklist.md.

use std::path::{Path, PathBuf};

use cpl_codegen::cpp_counts::{
    block_copies, container_copies, declared, entry_copies, lent_part_copies, map_get_copies,
    optional_copies, std_functions, sunk_copies,
};
#[path = "support/result_cache.rs"]
mod result_cache;
use result_cache::ResultCache;

/// Counts one pattern in generated C++.
type Count = fn(&str) -> usize;

const NEVER: [(&str, Count); 4] = [
    ("map_get_copies", map_get_copies),
    ("block_copies", block_copies),
    ("entry_copies", entry_copies),
    ("lent_part_copies", lent_part_copies),
];

/// What a program's header says its C++ makes: `// copies: N` deep copies,
/// `// optional_copies: N` containers copied into an optional,
/// `// std_functions: N` type-erased `std::function`s and
/// `// sunk_copies: N` places copied into a param a fn moves into its
/// struct, 0 when the line is absent. scripts/debt-ratchet.py bounds each
/// header sum over the corpus.
const DIRECTIVES: [(&str, Count); 4] = [
    ("copies", container_copies),
    ("optional_copies", optional_copies),
    ("std_functions", std_functions),
    ("sunk_copies", sunk_copies),
];

/// How `src`'s C++ disagrees with its header.
fn disagreements(src: &Path, out: &Path) -> Vec<String> {
    let text = std::fs::read_to_string(src).unwrap();
    let built = cpl_cli::run(&[
        "build",
        src.to_str().unwrap(),
        "--full",
        "-o",
        out.to_str().unwrap(),
    ]);
    let cpp = std::fs::read_to_string(out)
        .ok()
        .filter(|_| built.code == 0);
    let _ = std::fs::remove_file(out);
    let mut wrong = Vec::new();
    for (key, count) in DIRECTIVES {
        let want = match declared(&text, key) {
            Ok(n) => n,
            Err(e) => {
                wrong.push(e);
                continue;
            }
        };
        match &cpp {
            Some(cpp) if count(cpp) != want => wrong.push(format!(
                "the C++ makes {} {key}, the header says {want} (write `// {key}: {}`)",
                count(cpp),
                count(cpp)
            )),
            None if want > 0 => wrong.push(format!(
                "`// {key}: {want}` on a program that does not build"
            )),
            _ => {}
        }
    }
    if let Some(cpp) = &cpp {
        for (name, count) in NEVER {
            if count(cpp) > 0 {
                wrong.push(format!("{} {name}: a read made into a copy", count(cpp)));
            }
        }
        let copies = container_copies(cpp) + optional_copies(cpp) + sunk_copies(cpp);
        let explained = if copies > 0 { explained_copies(src) } else { 0 };
        if explained < copies {
            wrong.push(format!(
                "{copies} copies, `cpl explain` gives {explained} reasons"
            ));
        }
    }
    wrong
}

/// The `.cp` files one directory down from each root: the corpus
/// problems and the conformance cases.
fn sources() -> Vec<PathBuf> {
    cpl_corpus::programs::shipped(&repo())
}

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The shard of the program at `rel`, its path from the repo root: FNV-1a
/// of the path, mod `SHARDS`. Each shard is one test, so a change to one
/// program runs only its shard; scripts/affected.py computes the same.
fn shard_of(rel: &str) -> u64 {
    rel.bytes().fold(0xcbf2_9ce4_8422_2325, |h: u64, b| {
        (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
    }) % SHARDS
}

const SHARDS: u64 = 16;

fn rel(src: &Path) -> String {
    let rel = src.strip_prefix(repo()).unwrap_or(src);
    rel.to_string_lossy().replace('\\', "/")
}

/// Builds every program of shard `k` and checks its C++ against its header.
fn check_shard(k: u64) {
    let out = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("read_only_copies_{k}"));
    std::fs::create_dir_all(&out).unwrap();
    let sources: Vec<PathBuf> = sources()
        .into_iter()
        .filter(|src| shard_of(&rel(src)) == k)
        .collect();
    let cache = ResultCache::new(result_cache::default_dir("copies"), &repo());
    let wrong = disagreements_of(&sources, &out, &cache);
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// Each program's disagreements, `rel: why`, in `sources` order, built in
/// parallel; `cache` keeps them between runs.
fn disagreements_of(sources: &[PathBuf], out: &Path, cache: &ResultCache) -> Vec<String> {
    let next = std::sync::atomic::AtomicUsize::new(0);
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    std::thread::scope(|s| {
        let jobs: Vec<_> = (0..threads)
            .map(|n| {
                let (out, next) = (out.join(format!("{n}.cpp")), &next);
                s.spawn(move || {
                    let mut wrong = Vec::new();
                    while let Some(i) =
                        Some(next.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
                            .filter(|&i| i < sources.len())
                    {
                        let src = &sources[i];
                        let rel = rel(src);
                        let found = cache.lines(src, || disagreements(src, &out));
                        wrong.push((
                            i,
                            found
                                .into_iter()
                                .map(|w| format!("{rel}: {w}"))
                                .collect::<Vec<_>>(),
                        ));
                    }
                    wrong
                })
            })
            .collect();
        let mut all: Vec<_> = jobs.into_iter().flat_map(|j| j.join().unwrap()).collect();
        all.sort_by_key(|(i, _)| *i);
        all.into_iter().flat_map(|(_, w)| w).collect()
    })
}

/// A warm cache gives back exactly what a cold one computed, and an edit
/// recomputes only the edited program.
#[test]
fn cached_disagreements_match_a_cold_run_and_an_edit_recomputes_only_its_program() {
    let tmp =
        Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("copies_cache_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    let (cache_dir, progs) = (tmp.join("cache"), tmp.join("programs"));
    std::fs::create_dir_all(&progs).unwrap();
    // A shipped program with copies, the same with its count misstated (a
    // non-empty result) and one with none.
    let sample = sources_of_shard_sample();
    let base = std::fs::read_to_string(&sample[0]).unwrap();
    let n = declared(&base, "copies").unwrap();
    let wrong_header = base.replacen(&format!("copies: {n}"), &format!("copies: {}", n + 1), 1);
    let texts = [
        base.clone(),
        wrong_header,
        "print(read() + 1)\n".to_string(),
    ];
    let mut sources: Vec<PathBuf> = texts
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let p = progs.join(format!("p{i}.cp"));
            std::fs::write(&p, t).unwrap();
            p
        })
        .collect();
    sources.extend(sample);
    let out = tmp.join("out");
    std::fs::create_dir_all(&out).unwrap();
    let run = || {
        let cache = ResultCache::new(Some(cache_dir.clone()), &repo());
        let found = disagreements_of(&sources, &out, &cache);
        (found, cache.misses())
    };
    let uncached = disagreements_of(&sources, &out, &ResultCache::new(None, &repo()));
    let (cold, cold_misses) = run();
    assert_eq!(cold_misses, sources.len());
    assert_eq!(cold, uncached);
    let misstated = format!("p1.cp: the C++ makes {n} copies, the header says {}", n + 1);
    assert!(cold.iter().any(|w| w.contains(&misstated)), "{cold:?}");
    let (warm, warm_misses) = run();
    assert_eq!((warm_misses, &warm), (0, &cold));
    std::fs::write(&sources[1], &texts[0]).unwrap();
    let (edited, edited_misses) = run();
    assert_eq!(edited_misses, 1);
    assert_eq!(
        edited,
        disagreements_of(&sources, &out, &ResultCache::new(None, &repo()))
    );
    assert!(!edited.iter().any(|w| w.contains("p1.cp")), "{edited:?}");
    let _ = std::fs::remove_dir_all(&tmp);
}

/// A few shipped programs with copies, so the cached path covers `cpl explain`.
fn sources_of_shard_sample() -> Vec<PathBuf> {
    let mut with_copies: Vec<PathBuf> = sources()
        .into_iter()
        .filter(|s| declared(&std::fs::read_to_string(s).unwrap(), "copies").unwrap_or(0) > 0)
        .collect();
    with_copies.truncate(4);
    with_copies
}

macro_rules! shards {
    ($($name:ident = $k:literal),* $(,)?) => {
        $(#[test]
        fn $name() {
            check_shard($k)
        })*
        const SHARD_TESTS: &[u64] = &[$($k),*];
    };
}

shards!(
    shard_00 = 0,
    shard_01 = 1,
    shard_02 = 2,
    shard_03 = 3,
    shard_04 = 4,
    shard_05 = 5,
    shard_06 = 6,
    shard_07 = 7,
    shard_08 = 8,
    shard_09 = 9,
    shard_10 = 10,
    shard_11 = 11,
    shard_12 = 12,
    shard_13 = 13,
    shard_14 = 14,
    shard_15 = 15,
);

#[test]
fn every_shipped_program_falls_in_a_shard_with_a_test() {
    assert_eq!(SHARD_TESTS, (0..SHARDS).collect::<Vec<_>>());
    let sources = sources();
    assert!(
        sources.len() >= 1000,
        "only {} programs found",
        sources.len()
    );
    for k in 0..SHARDS {
        assert!(
            sources.iter().any(|src| shard_of(&rel(src)) == k),
            "shard {k} is empty"
        );
    }
    // scripts/test_affected.py pins the same paths to the same shards.
    assert_eq!(shard_of("spec/conformance/std-models/heaps.cp"), 8);
    assert_eq!(shard_of("corpus/problems/cf2165c/port.cp"), 4);
}

#[test]
fn explain_gives_one_reason_per_copy_a_library_call_makes() {
    let dir = std::env::temp_dir().join(format!("cpl-explain-lib-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let src = dir.join("bag.cp");
    std::fs::write(
        &src,
        concat!(
            "use std.bag.{AugSortedBag}\n",
            "var s: AugSortedBag<Int, Str> = AugSortedBag((x, y) => x + y, \"\", k => Str(k))\n",
            "s.add(3)\n",
            "let lo: Int = read()\n",
            "print(s.fold_between(lo, 9))\n",
        ),
    )
    .unwrap();
    let out = dir.join("bag.cpp");
    let built = cpl_cli::run(&[
        "build",
        src.to_str().unwrap(),
        "--full",
        "-o",
        out.to_str().unwrap(),
    ]);
    assert_eq!(built.code, 0);
    let cpp = std::fs::read_to_string(&out).unwrap();
    let copies = container_copies(&cpp) + optional_copies(&cpp);
    let explained = explained_copies(&src);
    let _ = std::fs::remove_dir_all(&dir);
    // fold_between and fold_of give the identity on an empty range by pointer
    // (`cpl::Held`), so the copies they made are gone, and explain lists none.
    assert!(cpp.contains("cpl::held_ref(self.id)"));
    assert_eq!(explained, copies);
}

#[test]
fn a_fn_whose_every_arm_returns_a_part_of_self_lends_it_and_explain_lists_no_copy() {
    for (case, lent) in [
        (
            "spec/conformance/std-models/augbag.cp",
            ["AugSortedBag.all"].as_slice(),
        ),
        (
            "spec/conformance/std-bag/aug-sorted-bag-set-match-brute.cp",
            &["AugSortedBag.all"],
        ),
        (
            "spec/conformance/std-segtree/right-while-lends-str-parts.cp",
            // `seen` keeps `v = c ? blank_at(..) : self.t[p]` by value, a copy
            // explain lists.
            &["SegTree.blank_at"],
        ),
    ] {
        let src = repo().join(case);
        let out = std::env::temp_dir().join(format!("cpl-lent-arms-{}.cpp", std::process::id()));
        let (src_s, out_s) = (src.to_str().unwrap(), out.to_str().unwrap());
        let built = cpl_cli::run(&["build", src_s, "--full", "-o", out_s]);
        assert_eq!(built.code, 0, "{case}");
        let cpp = std::fs::read_to_string(&out).unwrap();
        let _ = std::fs::remove_file(&out);
        let explain = cpl_cli::run(&["explain", src.to_str().unwrap()]);
        let explain = String::from_utf8_lossy(&explain.stdout);
        for f in lent {
            let (ty, name) = f.split_once('.').unwrap();
            let sigs: Vec<&str> = cpp
                .lines()
                .filter(|l| {
                    l.contains(&format!(" {ty}_")) && l.contains(&format!("_{name}(const {ty}_"))
                })
                .filter(|l| !l.starts_with("inline ll ") && !l.starts_with("inline bool "))
                .collect();
            assert!(
                !sigs.is_empty(),
                "{case}: no {f} returning a value in the C++"
            );
            for sig in sigs.iter().filter(|_| name != "seen") {
                assert!(
                    sig.starts_with("inline const ") && sig.contains("& "),
                    "{case}: {sig}"
                );
            }
            assert!(
                !explain
                    .lines()
                    .any(|l| l.contains("[copy]") && l.contains(&format!(" in {f} "))),
                "{case}: explain still lists a copy in {f}:\n{explain}"
            );
        }
    }
    let arms = "inline const std::vector<ll>& f(const S& self, ll a) {\n  if (a < 0) {\n    return self.id;\n  }\n  return a == 0 ? g(self) : cpl::at(self.agg, a);\n}\n";
    assert_eq!(container_copies(arms), 0);
}

#[test]
fn a_part_returned_by_value_is_a_copy_and_by_reference_is_not() {
    let by_value = "std::vector<ll> f(const S& self) {\n    return self.xs;\n}\n";
    let lent = "const std::vector<ll>& f(const S& self) {\n    return self.xs;\n}\n";
    let local = "std::vector<ll> f() {\n    std::vector<ll> xs;\n    return xs;\n}\n";
    assert_eq!(container_copies(by_value), 1);
    assert_eq!(container_copies(lent), 0);
    assert_eq!(container_copies(local), 0);
    let element = "std::string f(const S& self, ll t) {\n    return self.agg[t];\n}\n";
    assert_eq!(container_copies(element), 1);
    let after_lambda = "ll f(ll u) {\n  auto t = [&]() -> std::vector<ll> {\n    return v;\n  }();\n  return need[u];\n}\n";
    assert_eq!(container_copies(after_lambda), 0);
    let forwarded = "inline const std::vector<ll>& A_view(const A& self) {\n  return self.xs;\n}\ninline std::vector<ll> B_view(const B& self) {\n  return A_view(self.a);\n}\n";
    assert_eq!(container_copies(forwarded), 1);
    let cached = "inline std::vector<ll> T_plain(T& self) {\n  return self.xs;\n}\n";
    assert_eq!(container_copies(cached), 1);
}

#[test]
fn a_place_passed_to_a_fn_that_moves_it_into_a_struct_is_a_sunk_copy() {
    let cpp = "inline H wrap(std::vector<ll> xs) {\n  return H{std::move(xs)};\n}\nvoid solve() {\n  s = (*wrap(a).xs).size() + wrap(f(a)).xs.size() + wrap(std::move(a)).xs.size();\n}\n";
    assert_eq!(sunk_copies(cpp), 1);
    assert_eq!(
        sunk_copies(&cpp.replace("std::vector<ll> xs", "const std::vector<ll>& xs")),
        0
    );
    let pair = "struct P {\n  ll a = 0;\n  Q q;\n};\nstruct Q {\n  int b = 0;\n};\n";
    let flat = cpp.replace("std::vector<ll> xs", "P xs");
    assert_eq!(sunk_copies(&format!("{pair}{flat}")), 0);
    let held = pair.replace("int b = 0;", "std::vector<ll> b;");
    assert_eq!(sunk_copies(&format!("{held}{flat}")), 1);
    let commented = format!("{cpp}// fn f(a) = wrap(a)\n");
    assert_eq!(sunk_copies(&commented), 1);
    let small = "struct P {\n  ll x = 0;\n  bool b = false;\n};\nstruct W {\n  P p;\n  std::vector<ll> xs;\n};\n";
    let holding = |ty: &str| {
        format!("{small}inline H wrap({ty} v) {{\n  return H{{std::move(v)}};\n}}\nvoid solve() {{\n  h = wrap(a);\n}}\n")
    };
    assert_eq!(sunk_copies(&holding("P")), 0);
    assert_eq!(sunk_copies(&holding("W")), 1);
    let named =
        "struct S {\n  P p;\n  std::string name;\n};\nstruct M {\n  std::map<ll, ll> at;\n};\n";
    let held_in = |ty: &str| holding(ty).replacen("inline", &format!("{named}inline"), 1);
    assert_eq!(sunk_copies(&held_in("S")), 1);
    assert_eq!(sunk_copies(&held_in("M")), 1);
    assert_eq!(sunk_copies(&holding("std::string")), 1);
    let only_comment =
        format!("{small}inline H wrap(W v) {{\n  return H{{std::move(v)}};\n}}\n// h = wrap(a);\n");
    assert_eq!(sunk_copies(&only_comment), 0);
}

#[test]
fn a_header_states_a_count_once_and_above_zero() {
    assert_eq!(declared("// expect: ok\nprint(1)\n", "copies"), Ok(0));
    assert_eq!(
        declared("// expect: ok\n// copies: 2\nprint(1)\n", "copies"),
        Ok(2)
    );
    assert_eq!(declared("print(1)\n// copies: 2\n", "copies"), Ok(0));
    assert!(declared("// copies: 0\n", "copies").is_err());
    assert!(declared("// copies: two\n", "copies").is_err());
    assert!(declared("// copies: 1\n// copies: 1\n", "copies").is_err());
    assert_eq!(declared("// std_functions: 3\n", "std_functions"), Ok(3));
}

#[test]
fn a_yielded_tuple_built_from_rows_copies_them_and_one_of_references_does_not() {
    let pairs = "if (int cpl_r = cpl_f(std::tuple<std::vector<int>, std::vector<int>>{xs[i], xs[j]})) return cpl_r;";
    assert_eq!(container_copies(pairs), 1);
    let lent = "if (int cpl_r = cpl_f(std::forward_as_tuple(xs[i], xs[j]))) return cpl_r;";
    assert_eq!(container_copies(lent), 0);
}

#[test]
fn a_shortened_std_function_counts_at_each_use() {
    let cpp = "template<class...T>using Qy=std::function<T...>;\n\
               struct M { Qy<ll(ll, ll)> add; Qy<ll(ll, ll)> mul; };\n\
               // Qy<ll()> in a comment\n\
               std::function<void()> f; XQy<int> not_it;\n";
    assert_eq!(std_functions(cpp), 3);
}

/// The `[copy]` lines `cpl explain` prints for `src`: each copy the judge
/// build makes, with why it must own its value.
fn explained_copies(src: &Path) -> usize {
    let explain = cpl_cli::run(&["explain", src.to_str().unwrap()]);
    String::from_utf8_lossy(&explain.stdout)
        .lines()
        .filter(|l| l.trim_start().starts_with("[copy] "))
        .count()
}

/// Codegen's copy fact is the truth about which copies are required: a
/// copy `cpl explain --json` marks `kept` is one that rewriting its C++ to a
/// move or a borrow of what it copies changes the output of, and no other
/// copy is. Each spec/conformance case with a `// copies:` header is built
/// `--full`, each copy rewritten both ways, and the outputs compared.
#[test]
fn a_copy_is_kept_exactly_when_a_move_or_a_borrow_would_show() {
    let cases: Vec<PathBuf> = sources()
        .into_iter()
        .filter(|src| rel(src).starts_with("spec/conformance/"))
        .filter(|src| {
            declared(&std::fs::read_to_string(src).unwrap(), "copies").is_ok_and(|n| n > 0)
        })
        .collect();
    assert!(!cases.is_empty(), "no conformance case states a copy");
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("kept_copies");
    std::fs::create_dir_all(&dir).unwrap();
    let wrong: Vec<String> = std::thread::scope(|s| {
        let jobs: Vec<_> = cases
            .iter()
            .enumerate()
            .map(|(i, src)| {
                let dir = dir.join(i.to_string());
                s.spawn(move || {
                    std::fs::create_dir_all(&dir).unwrap();
                    let (kept, shown) = (kept_copies(src), shown_copies(src, &dir));
                    (kept != shown).then(|| {
                        format!(
                            "{}: `cpl explain` keeps {kept} copies, a move or a borrow shows {shown}",
                            rel(src)
                        )
                    })
                })
            })
            .collect();
        jobs.into_iter().filter_map(|j| j.join().unwrap()).collect()
    });
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// The copies `cpl explain --json` marks kept in `src`.
fn kept_copies(src: &Path) -> usize {
    let explain = cpl_cli::run(&["explain", "--json", src.to_str().unwrap()]);
    String::from_utf8_lossy(&explain.stdout)
        .matches("\"kept\":true")
        .count()
}

/// The copies of `src`'s `--full` C++ whose rewrite to a move or a borrow
/// of what they copy changes what the program prints on its `// input:`.
fn shown_copies(src: &Path, dir: &Path) -> usize {
    let cpp_path = dir.join("p.cpp");
    let built = cpl_cli::run(&[
        "build",
        "--full",
        src.to_str().unwrap(),
        "-o",
        cpp_path.to_str().unwrap(),
    ]);
    assert_eq!(built.code, 0, "{} does not build", rel(src));
    let cpp = std::fs::read_to_string(&cpp_path).unwrap();
    let input = case_input(&std::fs::read_to_string(src).unwrap());
    let base = run_cpp(&cpp, &input, dir).unwrap_or_else(|e| panic!("{}: {e}", rel(src)));
    let lines: Vec<&str> = cpp.lines().collect();
    let total = container_copies(&cpp);
    let mut sites = 0;
    let mut shown = 0;
    for (k, line) in lines.iter().enumerate() {
        let Some((moved, borrowed)) = rewrites(line) else {
            continue;
        };
        let with = |new: &str| {
            let mut out = lines.clone();
            out[k] = new;
            out.join("\n")
        };
        if container_copies(&with(&moved)) >= total {
            continue;
        }
        sites += 1;
        let differs = [moved, borrowed].iter().any(|new| {
            run_cpp(&with(new), &input, dir).unwrap_or_else(|e| panic!("{}: {e}", rel(src))) != base
        });
        shown += usize::from(differs);
    }
    assert_eq!(
        sites,
        total,
        "{}: found {sites} of its {total} copies",
        rel(src)
    );
    shown
}

/// `T name = place;` rewritten to move out of the place and to borrow it.
fn rewrites(line: &str) -> Option<(String, String)> {
    let body = line.trim_start().strip_suffix(';')?;
    let indent = &line[..line.len() - line.trim_start().len()];
    let (decl, place) = body.split_once(" = ")?;
    let (ty, name) = decl.rsplit_once(' ')?;
    let plain = |s: &str| !s.is_empty() && s.chars().all(|c| c.is_alphanumeric() || c == '_');
    if !plain(name) || ty.starts_with("const ") || ty == "auto" {
        return None;
    }
    Some((
        format!("{indent}{ty} {name} = std::move(const_cast<{ty}&>({place}));"),
        format!("{indent}{ty}& {name} = const_cast<{ty}&>({place});"),
    ))
}

/// What the C++ `cpp` prints on `input`, built in `dir`.
fn run_cpp(cpp: &str, input: &str, dir: &Path) -> Result<String, String> {
    use std::io::Write;
    let (src, bin) = (dir.join("v.cpp"), dir.join("v"));
    std::fs::write(&src, cpp).unwrap();
    let cc = std::process::Command::new("g++")
        .args(["-std=c++17", "-O1", "-w", "-o"])
        .arg(&bin)
        .arg(&src)
        .output()
        .map_err(|e| format!("g++: {e}"))?;
    if !cc.status.success() {
        return Err(String::from_utf8_lossy(&cc.stderr)
            .lines()
            .take(3)
            .collect());
    }
    let mut child = std::process::Command::new(&bin)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| e.to_string())?;
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    let out = child.wait_with_output().map_err(|e| e.to_string())?;
    Ok(format!(
        "{:?}\n{}",
        out.status.code(),
        String::from_utf8_lossy(&out.stdout)
    ))
}

/// A case's `// input: "..."` header, unescaped; empty when absent.
fn case_input(text: &str) -> String {
    let Some(quoted) = text
        .lines()
        .take_while(|l| l.starts_with("//"))
        .find_map(|l| l.strip_prefix("// input: \"")?.strip_suffix('"'))
    else {
        return String::new();
    };
    let mut out = String::new();
    let mut chars = quoted.chars();
    while let Some(c) = chars.next() {
        out.push(match (c, c == '\\') {
            (_, true) => match chars.next() {
                Some('n') => '\n',
                Some('t') => '\t',
                Some(other) => other,
                None => break,
            },
            (c, false) => c,
        });
    }
    out
}
