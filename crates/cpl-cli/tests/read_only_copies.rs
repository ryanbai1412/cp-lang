//! The C++ each corpus program and conformance case builds to makes exactly
//! the deep copies and `std::function`s its header states (`// copies: N`,
//! `// std_functions: N`; none when absent), so a new one turns the one
//! program that grew it red, by name. A heap value that is only read is read
//! in place: the copy shapes in `NEVER` fail anywhere, and `cpl explain`
//! gives each deep copy its reason.
//! Enforces performance/R12, R13 and R14 in
//! handbook/guidelines/performance-checklist.md.

use std::path::{Path, PathBuf};

#[path = "support/cpp_counts.rs"]
mod cpp_counts;
use cpp_counts::{deep_copies, is_place, optional_copies, std_functions, CONTAINERS};

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

/// The items of an argument or parameter list at depth 0.
fn items(list: &str) -> Vec<&str> {
    let (mut out, mut depth, mut start) = (Vec::new(), 0i32, 0);
    for (i, c) in list.char_indices() {
        match c {
            '(' | '<' | '[' | '{' => depth += 1,
            ')' | '>' | ']' | '}' => depth -= 1,
            ',' if depth == 0 => {
                out.push(list[start..i].trim());
                start = i + 1;
            }
            _ => {}
        }
    }
    if !list[start..].trim().is_empty() {
        out.push(list[start..].trim());
    }
    out
}

/// `inline H wrap(std::vector<ll> xs) { return H{std::move(xs)}; }` called
/// as `wrap(a)`: a fn takes a heap parameter by value to move it into the
/// struct it returns, and a call hands it a named place, which is copied.
/// A lent field points to its value instead (R05.4.4, R13.4).
fn sunk_copies(cpp: &str) -> usize {
    let lines: Vec<&str> = cpp.lines().collect();
    let head = |l: &str| -> Option<(String, String)> {
        let l = l.strip_prefix("inline ").unwrap_or(l).strip_suffix(") {")?;
        let (sig, params) = l.split_once('(')?;
        let name = sig.rsplit(' ').next()?;
        let named = sig.contains(' ') && name.chars().all(|c| c.is_alphanumeric() || c == '_');
        named.then(|| (name.to_string(), params.to_string()))
    };
    let mut sunk = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        let Some((name, params)) = head(line) else {
            continue;
        };
        let body = lines[i + 1..].iter().take_while(|l| **l != "}");
        let returns: Vec<&str> = body
            .map(|l| l.trim())
            .filter(|l| l.starts_with("return ") && l.contains('{'))
            .collect();
        for (k, param) in items(&params).into_iter().enumerate() {
            let Some((ty, p)) = param.rsplit_once(' ') else {
                continue;
            };
            let by_value = !ty.ends_with('&') && !ty.starts_with("[[") && !ty.starts_with('F');
            let heap = CONTAINERS.iter().any(|c| ty.starts_with(c))
                || ty.starts_with(|c: char| c.is_ascii_uppercase());
            let moved = format!("std::move({p})");
            if by_value && heap && returns.iter().any(|r| r.contains(&moved)) {
                sunk.push((name.clone(), k));
            }
        }
    }
    let mut count = 0;
    for line in lines.iter().filter(|l| head(l).is_none()) {
        for (name, k) in &sunk {
            let call = format!("{name}(");
            for (at, _) in line.match_indices(&call) {
                let before = line[..at].chars().next_back();
                if before.is_some_and(|c| c.is_alphanumeric() || c == '_') {
                    continue;
                }
                let rest = &line[at + call.len()..];
                let mut depth = 1;
                let end = rest.char_indices().find(|&(_, c)| {
                    depth += match c {
                        '(' => 1,
                        ')' => -1,
                        _ => 0,
                    };
                    depth == 0
                });
                let args = end.map_or(Vec::new(), |(e, _)| items(&rest[..e]));
                count += usize::from(args.get(*k).is_some_and(|a| is_place(a)));
            }
        }
    }
    count
}

/// Counts one pattern in generated C++.
type Count = fn(&str) -> usize;

/// Copy shapes no program may build to: each is a read made into a copy.
/// `std::vector<..> f(const S& self) {` then `return self.xs;`: a fn, like a
/// `@get` of a stored list, that copies out a part of what it was lent
/// instead of lending it on.
fn lent_part_copies(cpp: &str) -> usize {
    let lines: Vec<&str> = cpp.lines().collect();
    lines
        .windows(2)
        .filter(|w| {
            let sig = w[0].trim();
            let sig = sig.strip_prefix("inline ").unwrap_or(sig);
            let Some(part) = w[1].trim().strip_prefix("return ") else {
                return false;
            };
            let name: String = part
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
            cpp_counts::is_container(sig)
                && sig.ends_with(") {")
                && !name.is_empty()
                && part[name.len()..].starts_with(['.', '['])
                && part.ends_with(';')
                && [format!("& {name},"), format!("& {name})")]
                    .iter()
                    .any(|p| {
                        sig.find(p.as_str())
                            .is_some_and(|at| sig[..at].contains("const "))
                    })
        })
        .count()
}

const NEVER: [(&str, Count); 4] = [
    ("map_get_copies", map_get_copies),
    ("block_copies", block_copies),
    ("entry_copies", entry_copies),
    ("lent_part_copies", lent_part_copies),
];

/// What a program's header says its C++ makes: `// copies: N` deep copies,
/// `// optional_copies: N` containers copied into an optional and
/// `// std_functions: N` type-erased `std::function`s, 0 when the line is
/// absent.
const DIRECTIVES: [(&str, Count); 3] = [
    ("copies", deep_copies),
    ("optional_copies", optional_copies),
    ("std_functions", std_functions),
];

/// The most of each the corpus may build to, summed over programs.
const TOTALS: [(&str, Count, usize); 1] = [("sunk_copies", sunk_copies, 8)];

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
        let copies = deep_copies(cpp) + optional_copies(cpp);
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
    cpl_corpus::programs::shipped(&repo())
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
fn explain_gives_one_reason_per_copy_a_library_call_makes() {
    let src = repo().join("corpus/problems/nccc10s3/port.cp");
    let out = std::env::temp_dir().join(format!("cpl-explain-nccc-{}.cpp", std::process::id()));
    let built = cpl_cli::run(&["build", src.to_str().unwrap(), "-o", out.to_str().unwrap()]);
    assert_eq!(built.code, 0);
    let cpp = std::fs::read_to_string(&out).unwrap();
    let _ = std::fs::remove_file(&out);
    let copies = deep_copies(&cpp) + optional_copies(&cpp);
    assert!(copies > 0);
    assert_eq!(explained_copies(&src), copies);
}

#[test]
fn a_part_returned_by_value_is_a_copy_and_by_reference_is_not() {
    let by_value = "std::vector<ll> f(const S& self) {\n    return self.xs;\n}\n";
    let lent = "const std::vector<ll>& f(const S& self) {\n    return self.xs;\n}\n";
    let local = "std::vector<ll> f() {\n    std::vector<ll> xs;\n    return xs;\n}\n";
    assert_eq!(deep_copies(by_value), 1);
    assert_eq!(deep_copies(lent), 0);
    assert_eq!(deep_copies(local), 0);
    let element = "std::string f(const S& self, ll t) {\n    return self.agg[t];\n}\n";
    assert_eq!(deep_copies(element), 1);
    let after_lambda = "ll f(ll u) {\n  auto t = [&]() -> std::vector<ll> {\n    return v;\n  }();\n  return need[u];\n}\n";
    assert_eq!(deep_copies(after_lambda), 0);
    let forwarded = "inline const std::vector<ll>& A_view(const A& self) {\n  return self.xs;\n}\ninline std::vector<ll> B_view(const B& self) {\n  return A_view(self.a);\n}\n";
    assert_eq!(deep_copies(forwarded), 1);
    let cached = "inline std::vector<ll> T_plain(T& self) {\n  return self.xs;\n}\n";
    assert_eq!(deep_copies(cached), 1);
}

#[test]
fn a_place_passed_to_a_fn_that_moves_it_into_a_struct_is_a_sunk_copy() {
    let cpp = "inline H wrap(std::vector<ll> xs) {\n  return H{std::move(xs)};\n}\nvoid solve() {\n  s = (*wrap(a).xs).size() + wrap(f(a)).xs.size() + wrap(std::move(a)).xs.size();\n}\n";
    assert_eq!(sunk_copies(cpp), 1);
    assert_eq!(
        sunk_copies(&cpp.replace("std::vector<ll> xs", "const std::vector<ll>& xs")),
        0
    );
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
