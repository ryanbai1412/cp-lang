//! The C++ each corpus program and conformance case builds to makes exactly
//! the deep copies its header states (`// copies: N`; none when absent), so
//! a new copy turns the one program that grew it red, by name. The
//! `std::function`s are bounded over the whole corpus until they move to a
//! header the same way. A heap value that is only read is read
//! in place: the copy shapes in `NEVER` fail anywhere, and `cpl explain`
//! gives each deep copy its reason.
//! Enforces performance/R12, R13 and R14 in
//! handbook/guidelines/performance-checklist.md.

use std::path::{Path, PathBuf};

#[path = "support/corpus_programs.rs"]
mod corpus_programs;

#[path = "support/cpp_counts.rs"]
mod cpp_counts;
use cpp_counts::{deep_copies, std_functions};

/// `std::optional<std::vector<..>> t = cpl::map_get(m, k)`: the entry
/// copied out of the map to be read.
fn map_get_copies(cpp: &str) -> usize {
    cpp.lines()
        .filter(|l| l.contains("std::optional<std::vector<") && l.contains("= cpl::map_get("))
        .count()
}

/// `[&]() -> std::vector<..> { D_len = ..; return xs; }()`: a block that
/// sizes a domain from a list, then gives the list by value.
fn block_copies(cpp: &str) -> usize {
    let lines: Vec<&str> = cpp.lines().collect();
    (0..lines.len())
        .filter(|&i| lines[i].contains("[&]() -> std::vector<") && lines[i].ends_with('{'))
        .filter(|&i| {
            let mut j = i + 1;
            while lines.get(j).is_some_and(|l| {
                let l = l.trim();
                l.split(" = ")
                    .next()
                    .is_some_and(|lhs| lhs.ends_with("_len"))
            }) {
                j += 1;
            }
            j > i + 1
                && lines.get(j).is_some_and(|l| {
                    let l = l.trim();
                    l.strip_prefix("return ")
                        .and_then(|r| r.strip_suffix(';'))
                        .is_some_and(|v| v.chars().all(|c| c.is_alphanumeric() || c == '_'))
                })
        })
        .count()
}

/// `std::vector<ll>(cpl::map_or(m, k, d))`: a copy of an entry, made to
/// be changed.
fn entry_copies(cpp: &str) -> usize {
    cpp.lines().filter(|l| l.contains(">(cpl::map_or(")).count()
}

/// Counts one pattern in generated C++.
type Count = fn(&str) -> usize;

/// Copy shapes no program may build to: each is a read made into a copy.
const NEVER: [(&str, Count); 3] = [
    ("map_get_copies", map_get_copies),
    ("block_copies", block_copies),
    ("entry_copies", entry_copies),
];

/// What a program's header says its C++ makes: `// copies: N` deep copies,
/// 0 when the line is absent.
const DIRECTIVES: [(&str, Count); 1] = [("copies", deep_copies)];

/// The most `std::function`s the corpus may build to, summed over programs.
const TOTALS: [(&str, Count, usize); 1] = [("std_functions", std_functions, 79)];

/// The value of the header line `// <key>: N` (0 without one), or why the
/// header is malformed. The header is the file's leading `//` lines.
fn declared(text: &str, key: &str) -> Result<usize, String> {
    let mut found = None;
    for line in text.lines() {
        let Some(rest) = line.trim_start().strip_prefix("//") else {
            break;
        };
        let Some(v) = rest
            .trim_start()
            .strip_prefix(key)
            .and_then(|r| r.strip_prefix(':'))
        else {
            continue;
        };
        let n = v
            .trim()
            .parse::<usize>()
            .ok()
            .filter(|&n| n > 0)
            .ok_or_else(|| format!("`//{rest}`: write a count above 0, or drop the line for 0"))?;
        if found.replace(n).is_some() {
            return Err(format!("two `// {key}:` lines"));
        }
    }
    Ok(found.unwrap_or(0))
}

/// How `src`'s C++ disagrees with its header, and its count of each of
/// `TOTALS` (0 when it doesn't build).
fn disagreements(src: &Path, out: &Path) -> (Vec<String>, [usize; TOTALS.len()]) {
    let text = std::fs::read_to_string(src).unwrap();
    let built = cpl_cli::run(&["build", src.to_str().unwrap(), "-o", out.to_str().unwrap()]);
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
    let totals = TOTALS.map(|(_, count, _)| cpp.as_deref().map_or(0, count));
    if let Some(cpp) = &cpp {
        for (name, count) in NEVER {
            if count(cpp) > 0 {
                wrong.push(format!("{} {name}: a read made into a copy", count(cpp)));
            }
        }
        let copies = deep_copies(cpp);
        let explained = if copies > 0 { explained_copies(src) } else { 0 };
        if explained < copies {
            wrong.push(format!(
                "{copies} copies, `cpl explain` gives {explained} reasons"
            ));
        }
    }
    (wrong, totals)
}

/// The `.cp` files one directory down from each root: the corpus
/// problems and the conformance cases.
fn sources() -> Vec<PathBuf> {
    corpus_programs::shipped(&repo())
}

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn every_program_makes_the_copies_its_header_says() {
    let out = Path::new(env!("CARGO_TARGET_TMPDIR")).join("read_only_copies");
    std::fs::create_dir_all(&out).unwrap();
    let sources = sources();
    let next = std::sync::atomic::AtomicUsize::new(0);
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    let found: Vec<(Vec<String>, [usize; TOTALS.len()])> = std::thread::scope(|s| {
        let jobs: Vec<_> = (0..threads)
            .map(|n| {
                let (out, sources, next) = (out.join(format!("{n}.cpp")), &sources, &next);
                s.spawn(move || {
                    let mut found = Vec::new();
                    while let Some(src) =
                        sources.get(next.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
                    {
                        let (wrong, totals) = disagreements(src, &out);
                        let rel = src
                            .strip_prefix(repo())
                            .unwrap_or(src)
                            .display()
                            .to_string();
                        let mut lines: Vec<String> =
                            wrong.into_iter().map(|w| format!("{rel}: {w}")).collect();
                        for ((name, _, _), n) in TOTALS.iter().zip(totals) {
                            if n > 0 {
                                lines.push(format!("{rel}: {n} {name}"));
                            }
                        }
                        found.push((lines, totals));
                    }
                    found
                })
            })
            .collect();
        jobs.into_iter().flat_map(|j| j.join().unwrap()).collect()
    });
    assert!(
        sources.len() >= 1000,
        "only {} programs found",
        sources.len()
    );
    let mut wrong = Vec::new();
    for (k, (name, _, bound)) in TOTALS.iter().enumerate() {
        let total: usize = found.iter().map(|(_, t)| t[k]).sum();
        if total > *bound {
            let suffix = format!(" {name}");
            let offenders = found
                .iter()
                .flat_map(|(l, _)| l)
                .filter(|l| l.ends_with(&suffix));
            wrong.push(format!(
                "{name}: {total} in all, more than the {bound} allowed"
            ));
            wrong.extend(offenders.cloned());
        } else if total < *bound {
            eprintln!("{name}: {total} in all, under the bound of {bound}: lower it");
        }
    }
    let suffixes: Vec<String> = TOTALS
        .iter()
        .map(|(name, _, _)| format!(" {name}"))
        .collect();
    let headers: Vec<&String> = found
        .iter()
        .flat_map(|(l, _)| l)
        .filter(|l| !suffixes.iter().any(|s| l.ends_with(s.as_str())))
        .collect();
    wrong.extend(headers.iter().map(|l| l.to_string()));
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
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

/// The `[copy]` lines `cpl explain` prints for `src`: each copy the judge
/// build makes, with why it must own its value.
fn explained_copies(src: &Path) -> usize {
    let explain = cpl_cli::run(&["explain", src.to_str().unwrap()]);
    String::from_utf8_lossy(&explain.stdout)
        .lines()
        .filter(|l| l.trim_start().starts_with("[copy] "))
        .count()
}
