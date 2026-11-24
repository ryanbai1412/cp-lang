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

/// The structs whose fields hold no container or `std::function`, directly
/// or through another struct: passing one by value copies a few words.
fn flat_structs(lines: &[&str]) -> std::collections::BTreeSet<String> {
    let mut fields = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        let Some(name) = line
            .strip_prefix("struct ")
            .and_then(|l| l.strip_suffix(" {"))
        else {
            continue;
        };
        let body: Vec<&str> = lines[i + 1..]
            .iter()
            .take_while(|l| **l != "};")
            .map(|l| l.trim())
            .filter(|l| !l.starts_with("auto ") && !l.starts_with("bool operator"))
            .collect();
        fields.push((name.to_string(), body));
    }
    let mut flat = std::collections::BTreeSet::new();
    loop {
        let more: Vec<String> = fields
            .iter()
            .filter(|(name, _)| !flat.contains(name))
            .filter(|(_, body)| {
                body.iter().all(|f| {
                    let decl = f.split(" = ").next().unwrap_or(f);
                    let ty = decl.rsplit_once(' ').map_or(decl, |(ty, _)| ty);
                    !CONTAINERS.iter().any(|c| f.contains(c))
                        && !f.contains("std::function")
                        && (!ty.starts_with(|c: char| c.is_ascii_uppercase()) || flat.contains(ty))
                })
            })
            .map(|(name, _)| name.clone())
            .collect();
        if more.is_empty() {
            return flat;
        }
        flat.extend(more);
    }
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
    let flat = flat_structs(&lines);
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
                || ty.starts_with(|c: char| c.is_ascii_uppercase()) && !flat.contains(ty);
            let moved = format!("std::move({p})");
            if by_value && heap && returns.iter().any(|r| r.contains(&moved)) {
                sunk.push((name.clone(), k));
            }
        }
    }
    let mut count = 0;
    let code = |l: &&&str| head(l).is_none() && !l.trim_start().starts_with("//");
    for line in lines.iter().filter(code) {
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
/// `// optional_copies: N` containers copied into an optional,
/// `// std_functions: N` type-erased `std::function`s and
/// `// sunk_copies: N` places copied into a param a fn moves into its
/// struct, 0 when the line is absent. scripts/debt-ratchet.py bounds each
/// header sum over the corpus.
const DIRECTIVES: [(&str, Count); 4] = [
    ("copies", deep_copies),
    ("optional_copies", optional_copies),
    ("std_functions", std_functions),
    ("sunk_copies", sunk_copies),
];

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

/// How `src`'s C++ disagrees with its header.
fn disagreements(src: &Path, out: &Path) -> Vec<String> {
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
    let next = std::sync::atomic::AtomicUsize::new(0);
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    let wrong: Vec<String> = std::thread::scope(|s| {
        let jobs: Vec<_> = (0..threads)
            .map(|n| {
                let (out, sources, next) = (out.join(format!("{n}.cpp")), &sources, &next);
                s.spawn(move || {
                    let mut wrong = Vec::new();
                    while let Some(src) =
                        sources.get(next.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
                    {
                        let rel = rel(src);
                        wrong.extend(
                            disagreements(src, &out)
                                .into_iter()
                                .map(|w| format!("{rel}: {w}")),
                        );
                    }
                    wrong
                })
            })
            .collect();
        jobs.into_iter().flat_map(|j| j.join().unwrap()).collect()
    });
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
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
    let built = cpl_cli::run(&["build", src.to_str().unwrap(), "-o", out.to_str().unwrap()]);
    assert_eq!(built.code, 0);
    let cpp = std::fs::read_to_string(&out).unwrap();
    let copies = deep_copies(&cpp) + optional_copies(&cpp);
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
    assert_eq!(deep_copies(arms), 0);
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
