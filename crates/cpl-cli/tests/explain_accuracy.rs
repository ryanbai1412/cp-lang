//! `cpl explain` checked against the C++ it describes, over the corpus
//! and the conformance cases. Explain says each closure it calls
//! `static` is called without a `std::function`, names `cpl_fnN` for the
//! direct ones, and lists every copy ("every other heap read is a
//! reference"). Each claim is checked against the C++ itself, using the
//! counters the copy ratchet uses, so explain can't call a guess proven.
//! Enforces performance/R17 in handbook/guidelines/performance-checklist.md.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[path = "support/cpp_counts.rs"]
mod cpp_counts;
use cpp_counts::{deep_copies, is_container, is_place, optional_copies, std_functions, CONTAINERS};

/// The `.cp` files one directory down from the corpus problems and the
/// conformance cases.
fn sources() -> Vec<PathBuf> {
    cpl_corpus::programs::corpus_and_conformance(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."),
    )
}

/// The `"lowering"` and `"detail"` of each row of `cpl explain --json`.
fn rows(json: &str) -> Vec<(String, String)> {
    let json = cpl_json::parse(json).expect("cpl explain --json is JSON");
    let field = |row: &cpl_json::Value, key: &str| row.get(key)?.as_str().map(str::to_string);
    json.as_array()
        .unwrap_or_default()
        .iter()
        .filter_map(|row| Some((field(row, "lowering")?, field(row, "detail")?)))
        .collect()
}

/// Whether `cpp` names `ident` as a whole word.
fn names(cpp: &str, ident: &str) -> bool {
    cpp.match_indices(ident).any(|(i, _)| {
        !cpp[i + ident.len()..].starts_with(|c: char| c.is_alphanumeric() || c == '_')
    })
}

/// The deep copies `cpp` spells out, each source copy once though more
/// than one specialization may write it.
fn cpp_copies(cpp: &str) -> usize {
    let mut seen = std::collections::BTreeSet::new();
    let distinct: Vec<&str> = cpp.lines().filter(|l| seen.insert(l.trim())).collect();
    let distinct = distinct.join("\n");
    deep_copies(&distinct) + optional_copies(&distinct)
}

/// The lines of `cpp` outside the runtime's `namespace cpl { .. }`, whose
/// own helpers (`std::string(sv)` in a view's `to_str`) are not the
/// program's copies.
fn program_lines(cpp: &str) -> impl Iterator<Item = &str> {
    let mut runtime = false;
    cpp.lines().filter(move |line| {
        if line.starts_with("namespace cpl {") {
            runtime = true;
        } else if runtime && line.starts_with("}  // namespace cpl") {
            runtime = false;
            return false;
        }
        !runtime
    })
}

/// The structs that hold a container field, directly or through another
/// such struct. Mirrors `container_structs` in
/// scripts/specgate/explain_copies.py.
fn container_structs(cpp: &str) -> std::collections::BTreeSet<String> {
    let mut bodies: Vec<(String, Vec<&str>)> = Vec::new();
    let mut open = false;
    for line in cpp.lines().map(str::trim) {
        if let Some(name) = line
            .strip_prefix("struct ")
            .and_then(|r| r.strip_suffix('{'))
            .map(str::trim)
            .filter(|n| n.chars().all(|c| c.is_alphanumeric() || c == '_'))
        {
            bodies.push((name.to_string(), Vec::new()));
            open = true;
        } else if open && line.starts_with('}') {
            open = false;
        } else if open {
            bodies.last_mut().unwrap().1.push(line);
        }
    }
    let mut holders = std::collections::BTreeSet::new();
    let mut grew = true;
    while grew {
        grew = false;
        for (name, lines) in &bodies {
            let holds = lines.iter().any(|l| {
                CONTAINERS.iter().any(|c| l.starts_with(c))
                    || holders.contains(
                        l.split(' ')
                            .next()
                            .unwrap_or("")
                            .split('<')
                            .next()
                            .unwrap_or(""),
                    )
            });
            if !holders.contains(name) && holds {
                holders.insert(name.clone());
                grew = true;
            }
        }
    }
    holders
}

/// The type inside the `<..>` that opens `rest`, and what follows it.
fn template_arg(rest: &str) -> Option<(&str, &str)> {
    let mut depth = 1;
    let end = rest.char_indices().find_map(|(i, c)| {
        depth += match c {
            '<' => 1,
            '>' => -1,
            _ => 0,
        };
        (depth == 0).then_some(i)
    })?;
    Some((&rest[..end], &rest[end + 1..]))
}

/// An optional of a heap value copied from a place: declared from one
/// (`std::optional<std::string> t0 = best;`) or built from a tuple's field
/// (`std::optional<std::vector<char>>(std::get<0>(sp))`), and a struct
/// holding one, or a fixed array, wrapped as a `?T` argument
/// (`std::optional<H>(h2)`; a bare container there is [`optional_copies`]'s). Mirrors `OPTIONAL_*` in
/// scripts/specgate/explain_copies.py.
fn optional_place_copies(cpp: &str) -> usize {
    let holders = container_structs(cpp);
    let heap = |ty: &str| {
        CONTAINERS.iter().any(|c| ty.starts_with(c))
            || ty.starts_with("std::array<")
            || holders.contains(ty.split('<').next().unwrap_or(""))
    };
    program_lines(cpp)
        .map(str::trim)
        .map(|line| {
            let decl = line.strip_prefix("const ").unwrap_or(line);
            let declared = decl
                .strip_prefix("std::optional<")
                .and_then(template_arg)
                .is_some_and(|(ty, rest)| {
                    heap(ty)
                        && rest
                            .strip_suffix(';')
                            .and_then(|r| r.split_once(" = "))
                            .is_some_and(|(name, init)| {
                                is_place(name) && !name.contains('.') && is_place(init)
                            })
                });
            let from_field = line
                .match_indices("std::optional<")
                .filter_map(|(at, _)| template_arg(&line[at + "std::optional<".len()..]))
                .filter(|(ty, rest)| {
                    let held = (holders.contains(*ty) || ty.starts_with("std::array<"))
                        && rest
                            .strip_prefix('(')
                            .and_then(|r| r.split_once(')'))
                            .is_some_and(|(place, _)| is_place(place));
                    held || heap(ty)
                        && rest
                            .strip_prefix("(std::get<")
                            .and_then(|r| r.split_once(">("))
                            .and_then(|(n, r)| Some((n, r.split_once("))")?.0)))
                            .is_some_and(|(n, place)| {
                                n.chars().all(|c| c.is_ascii_digit()) && is_place(place)
                            })
                })
                .count();
            usize::from(declared) + from_field
        })
        .sum()
}

/// `self.agg[t]`, `self.m.id`: a part of a value, fields and indexes in
/// any order.
fn is_part(text: &str) -> bool {
    let start = text
        .find(['.', '['])
        .filter(|&at| at > 0)
        .unwrap_or(text.len());
    let (base, mut rest) = text.split_at(start);
    let word = |t: &str| {
        t.starts_with(|c: char| c.is_alphabetic() || c == '_')
            && t.chars().all(|c| c.is_alphanumeric() || c == '_')
    };
    if !word(base) || rest.is_empty() {
        return false;
    }
    while !rest.is_empty() {
        if let Some(r) = rest.strip_prefix('[') {
            let Some((index, after)) = r.split_once(']') else {
                return false;
            };
            if index.is_empty() || index.contains('[') {
                return false;
            }
            rest = after;
        } else if let Some(r) = rest.strip_prefix('.') {
            let end = r.find(['.', '[']).unwrap_or(r.len());
            if !word(&r[..end]) {
                return false;
            }
            rest = &r[end..];
        } else {
            return false;
        }
    }
    true
}

fn is_part_or_name(text: &str) -> bool {
    is_part(text) || is_indexed_place(text)
}

/// A part of a value given out by value: `return self.agg[t];` in a fn
/// returning a heap type (a container's is [`deep_copies`]'s), `Mat v = c ?
/// f(..) : self.t[p];` and a tuple's field `std::vector<u8> w =
/// std::get<1>(top);`. Mirrors
/// `returned_parts` in scripts/specgate/explain_copies.py.
fn returned_part_copies(cpp: &str) -> usize {
    let holders = container_structs(cpp);
    let heap = |ty: &str| {
        CONTAINERS.iter().any(|c| ty.starts_with(c))
            || ty.starts_with("std::array<")
            || holders.contains(ty)
    };
    let (mut count, mut depth, mut scopes) = (0, 0i64, Vec::<(i64, Option<&str>)>::new());
    for line in program_lines(cpp) {
        let trimmed = line.trim();
        let lead = (trimmed.len() - trimmed.trim_start_matches('}').len()) as i64;
        depth -= lead;
        while scopes.last().is_some_and(|&(d, _)| d >= depth) {
            scopes.pop();
        }
        if !line.starts_with([' ', '}', '#', '/']) && line.ends_with('{') && line.contains('(') {
            let head = line.split_once('(').map_or("", |(h, _)| h);
            let head = match head.strip_prefix("template <") {
                Some(t) => t.split_once("> ").map_or("", |(_, h)| h),
                None => head,
            };
            let head = head
                .trim_start_matches("inline ")
                .trim_start_matches("static ");
            let ret = head
                .rsplit_once(' ')
                .map(|(ty, _)| ty.strip_prefix("const ").unwrap_or(ty));
            scopes.push((depth, ret));
        } else if trimmed.ends_with('{') && trimmed.contains("](") {
            let ret = trimmed
                .rsplit_once(") -> ")
                .map(|(_, r)| r.trim_end_matches('{').trim());
            scopes.push((depth, ret));
        }
        depth +=
            (trimmed.matches('{').count() as i64) - (trimmed.matches('}').count() as i64) + lead;
        let ret = scopes.last().and_then(|&(_, r)| r);
        let line = line.trim();
        let returned = line
            .strip_prefix("return ")
            .and_then(|r| r.strip_suffix(';'))
            .is_some_and(|e| {
                ret.is_some_and(|r| heap(r) && !is_container(r) && !r.ends_with(['&', '*']))
                    && is_part(e)
            });
        let decl = line.strip_prefix("const ").unwrap_or(line);
        let branch = decl.split_once(' ').is_some_and(|(ty, rest)| {
            ty.matches('<').count() == ty.matches('>').count()
                && heap(ty)
                && rest.contains(" = ")
                && rest.contains(" ? ")
                && rest
                    .strip_suffix(';')
                    .and_then(|r| r.rsplit_once(" : "))
                    .is_some_and(|(_, part)| is_part(part))
        });
        let field = decl.split_once(" = std::get<").is_some_and(|(head, rest)| {
            CONSTRUCTED.iter().any(|c| head.starts_with(c))
                && rest
                    .split_once(">(")
                    .and_then(|(n, r)| Some((n, r.strip_suffix(");")?)))
                    .is_some_and(|(n, place)| {
                        n.chars().all(|c| c.is_ascii_digit()) && is_part_or_name(place)
                    })
        });
        count += usize::from(returned) + usize::from(branch) + usize::from(field);
    }
    count
}

/// A fixed array captured by value under a new name, `[wt_2 = wt]`: the
/// closure holds its own array. Mirrors `ARRAY` in
/// scripts/specgate/explain_copies.py.
fn array_capture_copies(cpp: &str) -> usize {
    let lines: Vec<&str> = program_lines(cpp).collect();
    let mut arrays = HashSet::new();
    for line in &lines {
        for (at, _) in line.match_indices("std::array<") {
            let Some((_, rest)) = template_arg(&line[at + "std::array<".len()..]) else {
                continue;
            };
            let rest = rest.strip_prefix('&').unwrap_or(rest);
            let Some(rest) = rest.strip_prefix(' ') else {
                continue;
            };
            let end = rest
                .find(|c: char| !(c.is_alphanumeric() || c == '_'))
                .unwrap_or(rest.len());
            let (name, after) = rest.split_at(end);
            if !name.is_empty() && after.trim_start().starts_with(['=', ';', ',', ')']) {
                arrays.insert(name);
            }
        }
    }
    lines
        .iter()
        .flat_map(|line| line.split('[').skip(1))
        .filter_map(|tail| tail.split_once("]("))
        .flat_map(|(caps, _)| caps.split(','))
        .filter(|cap| {
            cap.split_once('=').is_some_and(|(name, src)| {
                !name.trim().starts_with('&') && arrays.contains(src.trim())
            })
        })
        .count()
}

/// The containers [`constructed_copies`] sees built from another value.
const CONSTRUCTED: [&str; 8] = [
    "std::vector<",
    "std::array<",
    "std::map<",
    "std::set<",
    "std::unordered_map<",
    "std::deque<",
    "cpl::Grid<",
    "std::string",
];

/// `xs`, `w[i]`, `self.rows`: a name, then indexes, then fields.
fn is_indexed_place(text: &str) -> bool {
    let word = |t: &str| {
        t.starts_with(|c: char| c.is_alphabetic() || c == '_')
            && t.chars().all(|c| c.is_alphanumeric() || c == '_')
    };
    let (head, fields) = text.split_once('.').unwrap_or((text, ""));
    let (base, index) = head.split_once('[').unwrap_or((head, ""));
    let index_ok = index.is_empty()
        || index.ends_with(']')
            && index[..index.len() - 1]
                .split("][")
                .all(|i| !i.is_empty() && !i.contains(['[', ']']));
    word(base) && index_ok && (fields.is_empty() || fields.split('.').all(word))
}

/// The argument of the call whose `(` opens `rest`, when it is a place.
fn place_arg(rest: &str) -> bool {
    let Some(arg) = rest.strip_prefix('(') else {
        return false;
    };
    if let Some(at) = arg.strip_prefix("cpl::at(") {
        return at
            .split_once(')')
            .is_some_and(|(inner, after)| !inner.contains('(') && after.starts_with(')'));
    }
    arg.split_once(')')
        .is_some_and(|(a, after)| is_indexed_place(a) && !after.starts_with('>'))
}

/// A second heap value spelled as a call: a container constructed from a
/// place (`std::array<ll, 4>(a)`, `std::set<ll>(a)`), `cpl::sorted(xs)`
/// (which takes its list by value), `cpl::to_list(..)` and `cpl::to_str(..)`. Mirrors `COPY`
/// in scripts/specgate/explain_copies.py. Counted here on top of
/// [`deep_copies`], not under the `// copies:` headers.
fn constructed_copies(cpp: &str) -> usize {
    program_lines(cpp)
        .map(|line| {
            let sorted = line
                .match_indices("cpl::sorted(")
                .filter(|(at, _)| {
                    line[at + "cpl::sorted(".len()..]
                        .split_once(')')
                        .is_some_and(|(a, _)| is_place(a) && !a.starts_with("cpl::"))
                })
                .count();
            let built = CONSTRUCTED
                .iter()
                .flat_map(|ty| line.match_indices(ty).map(move |(at, _)| at + ty.len()))
                .filter(|&start| {
                    let rest = &line[start..];
                    if !line[..start].ends_with('<') {
                        return place_arg(rest);
                    }
                    let mut depth = 1;
                    rest.char_indices()
                        .find_map(|(i, c)| {
                            depth += match c {
                                '<' => 1,
                                '>' => -1,
                                ';' | '(' | ')' => return Some(None),
                                _ => 0,
                            };
                            (depth == 0).then_some(Some(i))
                        })
                        .flatten()
                        .is_some_and(|end| place_arg(&rest[end + 1..]))
                })
                .count();
            line.matches("cpl::to_list(").count()
                + line.matches("cpl::to_str(").count()
                + sorted
                + built
        })
        .sum()
}

/// Each way explain's report on `src` disagrees with the C++ it builds.
fn disagreements(src: &Path, out: &Path) -> Option<Vec<String>> {
    let built = cpl_cli::run(&["build", src.to_str()?, "-o", out.to_str()?]);
    let cpp = std::fs::read_to_string(out).ok();
    let _ = std::fs::remove_file(out);
    let cpp = cpp.filter(|_| built.code == 0)?;
    let explained = cpl_cli::run(&["explain", src.to_str()?, "--json"]);
    if explained.code != 0 {
        return Some(vec![format!(
            "builds, but `cpl explain` fails: {}",
            explained.stderr
        )]);
    }
    let rows = rows(&String::from_utf8_lossy(&explained.stdout));
    let mut wrong = Vec::new();
    let dynamic = rows
        .iter()
        .filter(|(k, _)| k == "dynamic_std_function")
        .count();
    let functions = std_functions(&cpp);
    if dynamic == 0 && functions > 0 {
        wrong.push(format!(
            "explain lists no std::function, the C++ declares {functions}"
        ));
    }
    if dynamic > 0 && functions == 0 {
        wrong.push(format!(
            "explain lists {dynamic} std::functions, the C++ declares none"
        ));
    }
    for (_, closure) in rows.iter().filter(|(k, _)| k == "static_direct") {
        if !names(&cpp, closure) {
            wrong.push(format!(
                "explain says a call goes to {closure}, the C++ has none"
            ));
        }
    }
    let listed = rows.iter().filter(|(k, _)| k == "copy").count();
    let copies = cpp_copies(&cpp);
    if copies > listed {
        wrong.push(format!(
            "explain lists {listed} copies, the C++ spells out {copies}"
        ));
    }
    Some(wrong)
}

/// The programs split into this many tests, so nextest spreads them over
/// CI's test partitions instead of one job running them all.
const SHARDS: usize = 8;

fn explain_claims_on_shard(shard: usize) {
    let out = Path::new(env!("CARGO_TARGET_TMPDIR")).join("explain_accuracy");
    std::fs::create_dir_all(&out).unwrap();
    let sources: Vec<PathBuf> = sources().into_iter().skip(shard).step_by(SHARDS).collect();
    let next = std::sync::atomic::AtomicUsize::new(0);
    let checked = std::sync::atomic::AtomicUsize::new(0);
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    let wrong: Vec<String> = std::thread::scope(|s| {
        let jobs: Vec<_> = (0..threads)
            .map(|n| {
                let out = out.join(format!("{shard}-{n}.cpp"));
                let (sources, next, checked) = (&sources, &next, &checked);
                s.spawn(move || {
                    let mut wrong = Vec::new();
                    while let Some(src) =
                        sources.get(next.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
                    {
                        let Some(found) = disagreements(src, &out) else {
                            continue;
                        };
                        checked.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        wrong.extend(found.into_iter().map(|w| format!("{}: {w}", src.display())));
                    }
                    wrong
                })
            })
            .collect();
        jobs.into_iter().flat_map(|j| j.join().unwrap()).collect()
    });
    let checked = checked.into_inner();
    assert!(checked >= 1000 / SHARDS, "only {checked} programs built");
    assert!(
        wrong.is_empty(),
        "{} disagreements:\n{}",
        wrong.len(),
        wrong.join("\n")
    );
}

#[test]
fn explain_claims_only_what_the_cpp_does_0() {
    explain_claims_on_shard(0);
}

#[test]
fn explain_claims_only_what_the_cpp_does_1() {
    explain_claims_on_shard(1);
}

#[test]
fn explain_claims_only_what_the_cpp_does_2() {
    explain_claims_on_shard(2);
}

#[test]
fn explain_claims_only_what_the_cpp_does_3() {
    explain_claims_on_shard(3);
}

#[test]
fn explain_claims_only_what_the_cpp_does_4() {
    explain_claims_on_shard(4);
}

#[test]
fn explain_claims_only_what_the_cpp_does_5() {
    explain_claims_on_shard(5);
}

#[test]
fn explain_claims_only_what_the_cpp_does_6() {
    explain_claims_on_shard(6);
}

#[test]
fn explain_claims_only_what_the_cpp_does_7() {
    explain_claims_on_shard(7);
}

/// The copies `cpl explain --json` lists for `src`, and the ones the C++
/// `cpl build --full` writes for it makes.
fn listed_and_made(src: &Path, out: &Path) -> (usize, usize) {
    let built = cpl_cli::run(&[
        "build",
        src.to_str().unwrap(),
        "--full",
        "-o",
        out.to_str().unwrap(),
    ]);
    assert_eq!(built.code, 0, "{}: {}", src.display(), built.stderr);
    let cpp = std::fs::read_to_string(out).unwrap();
    let explained = cpl_cli::run(&["explain", src.to_str().unwrap(), "--json"]);
    assert_eq!(explained.code, 0, "{}: {}", src.display(), explained.stderr);
    let rows = rows(&String::from_utf8_lossy(&explained.stdout));
    let listed = rows.iter().filter(|(k, _)| k == "copy").count();
    (
        listed,
        deep_copies(&cpp)
            + optional_copies(&cpp)
            + constructed_copies(&cpp)
            + optional_place_copies(&cpp)
            + array_capture_copies(&cpp)
            + returned_part_copies(&cpp),
    )
}

#[test]
fn explain_lists_the_copies_the_judge_cpp_makes() {
    let conformance = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../spec/conformance");
    let out = Path::new(env!("CARGO_TARGET_TMPDIR")).join("explain_counts");
    std::fs::create_dir_all(&out).unwrap();
    for case in CASES {
        let src = conformance.join(case);
        let (listed, made) = listed_and_made(&src, &out.join("a.cpp"));
        assert_eq!(
            listed, made,
            "{case}: explain lists {listed} copies, the C++ makes {made}"
        );
    }
}

/// Cases whose copies explain once miscounted, against the C++ for the
/// judge.
const CASES: &[&str] = &[
    "std-lazy/rope-matches-brute.cp",
    "std-lazy/rope-lazy-treap-to-array-matches-brute.cp",
    "std-hashmap/reserve-keeps-entries.cp",
    "std-poly/advanced-free-fns-match-brute.cp",
    "core-01/while-let-tuple-pattern.cp",
    "core-05/exercise-34-loop-writes-separate-clone.cp",
    "std-models/dp-small.cp",
    "std-models/link-cut.cp",
    "std-models/historic-beats.cp",
    "std-models/euler-tour-tree.cp",
    "std-seq/set-union-intersect-subtract.cp",
    "core-04/grid-cols-of-one-row.cp",
    "core-12/each-types-generic-folds.cp",
    "std-matrix/bitmat.cp",
    "std-strings/kmp-rolling-hash61-lcs-all.cp",
    "std-flow/circulation-infeasible-none.cp",
    "std-flow/circulation-zero-lower-bounds.cp",
    "std-flow/matroid-intersection-and-min-cost-circulation-match-brute.cp",
    "std-matrix/min-plus-gaps-fold-in-a-segtree.cp",
    "traits/Show/Trie--show.cp",
    "core-01/if-is-branch-view-meets-owned-else.cp",
    "core-04/spec-block-split-by-search.cp",
    "core-05/file-body-var-holds-lent.cp",
];

#[test]
fn an_optional_of_a_heap_value_from_a_place_is_a_copy() {
    let cpp = "struct H {\n  std::vector<ll> pre;\n};\nstruct G {\n  H h;\n};\n";
    let copies = [
        "std::optional<std::string> t0 = best;",
        "std::optional<G> t0 = of;",
        "return std::optional<std::vector<char>>(std::get<0>(sp));",
        "ll t8 = f(h, std::optional<H>(h2));",
        "return std::optional<std::array<ll, 5>>(std::get<1>(r));",
        "ll t8 = f(std::optional<std::array<ll, 2>>(a));",
    ];
    for line in copies {
        assert_eq!(
            optional_place_copies(&format!("{cpp}{line}\n")),
            1,
            "{line}"
        );
    }
    let not_copies = [
        "std::optional<ll> t0 = best;",
        "std::optional<std::string> t0 = f(best);",
        "return std::optional<ll>(std::get<0>(sp));",
        "ll t8 = f(h, std::optional<std::vector<ll>>(h2));",
        "namespace cpl {\nstd::optional<std::string> t0 = best;\n}  // namespace cpl",
    ];
    for line in not_copies {
        assert_eq!(
            optional_place_copies(&format!("{cpp}{line}\n")),
            0,
            "{line}"
        );
    }
}

#[test]
fn a_part_given_out_by_value_is_a_copy() {
    let cpp = "struct H {\n  std::vector<ll> pre;\n};\n";
    let copies = [
        "inline H all(const B& self) {\n  return self.agg[self.bag.root];\n}",
        "inline H blank(const T& self) {\n  return self.m.id;\n}",
        "  H v = p == 0 ? f(self) : self.t[p];",
        "  std::vector<cpl::u8> w = std::get<1>(top);",
    ];
    for line in copies {
        assert_eq!(returned_part_copies(&format!("{cpp}{line}\n")), 1, "{line}");
    }
    let not_copies = [
        "inline ll at(const T& self) {\n  return self.t[0];\n}",
        "inline H make(const T& self) {\n  return acc;\n}",
        "inline std::string& at(T& self) {\n  return self.vals[0];\n}",
        "inline H make(const T& s) {\n  auto f = [&](ll i) -> ll {\n    return s.t[i];\n  };\n}",
        "  ll v = p == 0 ? f(self) : self.t[p];",
        "std::add_pointer_t<std::vector<ll>(ll)> leaf;",
    ];
    for line in not_copies {
        assert_eq!(returned_part_copies(&format!("{cpp}{line}\n")), 0, "{line}");
    }
    assert_eq!(
        constructed_copies("std::add_pointer_t<std::vector<ll>(ll)> leaf;\n"),
        0
    );
}

#[test]
fn a_fixed_array_captured_by_value_is_a_copy() {
    let cpp = "  std::array<ll, 7> wt = std::move(t11);\n";
    let copy = "  auto f = [wt_2 = wt](int e) mutable -> ll { return wt_2[e]; };\n";
    assert_eq!(array_capture_copies(&format!("{cpp}{copy}")), 1);
    for line in [
        "  auto f = [&wt_2 = wt](int e) -> ll { return wt_2[e]; };\n",
        "  auto f = [n_2 = n](int e) -> ll { return e + n_2; };\n",
        "  auto f = [&](int e) -> ll { return wt[e]; };\n",
    ] {
        assert_eq!(array_capture_copies(&format!("{cpp}{line}")), 0, "{line}");
    }
}

#[test]
fn a_container_built_from_a_place_or_sorted_is_a_copy() {
    let copies = [
        "a2 = std::array<ll, 4>(a);",
        "u = std::set<ll>(a);",
        "std::vector<ll> want_2 = std::vector<ll>(sums);",
        "g = std::vector<std::vector<ll>>(rows[i]);",
        "auto s = std::string(self.name);",
        "m = std::map<ll, std::vector<ll>>(cpl::at(ms, i));",
        "a = cpl::sorted(a);",
        "show_1(cpl::sorted(self.xs), 10);",
        "auto v = cpl::to_list(it);",
        "return cpl::to_str(cpl::str_slice(s, 0, q));",
    ];
    for line in copies {
        assert_eq!(constructed_copies(line), 1, "{line}");
    }
    let not_copies = [
        "std::vector<ll> xs = std::vector<ll>();",
        "auto v = std::vector<ll>(s.begin(), s.end());",
        "auto v = std::vector<ll>(5);",
        "auto t = cpl::sorted(f(a));",
        "std::vector<std::string> names;",
        "auto v = std::vector<ll>{k};",
        "namespace cpl {\nstd::string to_str() const { return std::string(sv); }\n}  // namespace cpl",
    ];
    for line in not_copies {
        assert_eq!(constructed_copies(line), 0, "{line}");
    }
}
