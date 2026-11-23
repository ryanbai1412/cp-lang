//! A ratchet on the C++ the corpus and the conformance cases build to: a
//! heap value that is only read is read in place, never copied. Each
//! pattern below is a copy the generated code used to make, or (for
//! `deep_copies`) any deep copy of a container; its count may only go down.

use std::path::{Path, PathBuf};

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

/// A copy kind: its name, its counter over a C++ file, and its bound.
type Bound = (&'static str, fn(&str) -> usize, usize);

/// `std::vector<ll>(cpl::map_or(m, k, d))`: a copy of an entry, made to
/// be changed.
fn entry_copies(cpp: &str) -> usize {
    cpp.lines().filter(|l| l.contains(">(cpl::map_or(")).count()
}

/// The C++ container types a copy of which is deep.
const CONTAINERS: [&str; 6] = [
    "std::vector<",
    "std::map<",
    "std::set<",
    "std::unordered_map<",
    "std::deque<",
    "std::string",
];

/// A name or a place on one: `xs`, `self.rows`, `cpl::at(rows, i)`.
fn is_place(text: &str) -> bool {
    let text = text.trim();
    let name = |t: &str| {
        !t.is_empty()
            && t.split('.').all(|p| {
                p.starts_with(|c: char| c.is_alphabetic() || c == '_')
                    && p.chars().all(|c| c.is_alphanumeric() || c == '_')
            })
    };
    name(text) || (text.starts_with("cpl::at(") && text.ends_with(')') && !text[8..].contains("::"))
}

/// Every deep copy of a container the C++ spells out: a container local
/// initialised from a place (`std::vector<ll> row = cpl::at(rows, i);`), a
/// tuple of containers built from a pair's fields, and a by-value lambda
/// capture of a name declared as a container.
fn deep_copies(cpp: &str) -> usize {
    let container = |ty: &str| CONTAINERS.iter().any(|c| ty.starts_with(c));
    let mut held = std::collections::BTreeSet::new();
    let mut count = 0;
    for line in cpp.lines() {
        let line = line.trim();
        let decl = line.strip_prefix("const ").unwrap_or(line);
        if container(decl) {
            // `T name = place;`, `const T& name`, `T name(...)`: the name is
            // the last word before `=`, `(`, `;`, `,` or `)` at depth 0.
            let (mut depth, mut end) = (0i32, decl.len());
            for (i, c) in decl.char_indices() {
                match c {
                    '<' => depth += 1,
                    '>' => depth -= 1,
                    '=' | '(' | ';' | ',' | ')' | '{' if depth == 0 => {
                        end = i;
                        break;
                    }
                    _ => {}
                }
            }
            let head = decl[..end].trim_end();
            let name = head.rsplit([' ', '&']).next().unwrap_or("");
            if !name.is_empty() && name.len() < head.len() {
                held.insert(name.to_string());
                let by_value = !head[..head.len() - name.len()].trim_end().ends_with('&');
                let rhs = decl[end..]
                    .strip_prefix('=')
                    .and_then(|r| r.strip_suffix(';'));
                if by_value && rhs.is_some_and(is_place) {
                    count += 1;
                }
            }
        }
        if line.starts_with("std::tuple<")
            && CONTAINERS.iter().any(|c| line.contains(c))
            && line.contains(".first, ")
            && line.contains(".second)")
        {
            count += 1;
        }
    }
    for line in cpp.lines() {
        let Some(open) = line.find("= [") else {
            continue;
        };
        let Some(close) = line[open..].find("](") else {
            continue;
        };
        count += line[open + 3..open + close]
            .split(", ")
            .filter(|cap| !cap.starts_with('&'))
            .filter_map(|cap| cap.split_once(" = "))
            .filter(|(_, outer)| held.contains(outer.trim()))
            .count();
    }
    count
}

/// The most copies of each kind the corpus may build to, with why the
/// ones left are left.
/// Each `std::function` the C++ declares: a fn value whose callee codegen
/// could not pin down, called indirectly.
fn std_functions(cpp: &str) -> usize {
    cpp.lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .map(|l| l.matches("std::function<").count())
        .sum()
}

const BOUNDS: [Bound; 5] = [
    // `var cols = m.get(r) ?? []` then pushed to: a copy it needs.
    ("map_get_copies", map_get_copies, 1),
    ("block_copies", block_copies, 0),
    ("entry_copies", entry_copies, 0),
    // Measured when the ratchet began: snapshots taken before a change
    // (`old_s = self.state`) and temporaries copied out of places.
    ("deep_copies", deep_copies, 51),
    ("std_functions", std_functions, 610),
];

/// The `.cp` files one directory down from each root: the corpus
/// problems and the conformance cases.
fn sources() -> Vec<PathBuf> {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut found = Vec::new();
    for root in ["corpus/problems", "spec/conformance"] {
        for dir in std::fs::read_dir(repo.join(root)).expect("source root exists") {
            let dir = dir.unwrap().path();
            if !dir.is_dir() {
                continue;
            }
            for file in std::fs::read_dir(dir).unwrap() {
                let path = file.unwrap().path();
                if path.extension().is_some_and(|ext| ext == "cp") {
                    found.push(path);
                }
            }
        }
    }
    found.sort();
    found
}

#[test]
fn the_corpus_and_conformance_never_copy_a_heap_value_they_only_read() {
    let out = Path::new(env!("CARGO_TARGET_TMPDIR")).join("read_only_copies");
    std::fs::create_dir_all(&out).unwrap();
    let sources = sources();
    let chunk = sources.len().div_ceil(8);
    let built = std::sync::atomic::AtomicUsize::new(0);
    let found: Vec<(String, [usize; BOUNDS.len()])> = std::thread::scope(|s| {
        let jobs: Vec<_> = sources
            .chunks(chunk)
            .enumerate()
            .map(|(n, part)| {
                let (out, built) = (out.join(format!("{n}.cpp")), &built);
                s.spawn(move || {
                    let mut found = Vec::new();
                    for src in part {
                        let built_now = cpl_cli::run(&[
                            "build",
                            src.to_str().unwrap(),
                            "-o",
                            out.to_str().unwrap(),
                        ]);
                        let Ok(cpp) = std::fs::read_to_string(&out) else {
                            continue;
                        };
                        let _ = std::fs::remove_file(&out);
                        if built_now.code != 0 {
                            continue;
                        }
                        built.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        let counts = BOUNDS.map(|(_, count, _)| count(&cpp));
                        if counts.iter().any(|&c| c > 0) {
                            found.push((src.display().to_string(), counts));
                        }
                    }
                    found
                })
            })
            .collect();
        jobs.into_iter().flat_map(|j| j.join().unwrap()).collect()
    });
    let built = built.into_inner();
    assert!(
        built >= 1000,
        "only {built} corpus and conformance programs built"
    );
    for (k, (name, _, bound)) in BOUNDS.iter().enumerate() {
        let total: usize = found.iter().map(|(_, c)| c[k]).sum();
        let offenders: Vec<String> = found
            .iter()
            .filter(|(_, c)| c[k] > 0)
            .map(|(src, c)| format!("{src}: {}", c[k]))
            .collect();
        assert!(
            total <= *bound,
            "{name}: {total} copies, more than the {bound} allowed:\n{}",
            offenders.join("\n")
        );
        if total < *bound {
            eprintln!("{name}: {total} copies, under the bound of {bound}: lower it");
        }
    }
}
