//! Each std claim that a fn allocates nothing, checked on the VM: the fn
//! runs `m` and then `4m` times, and the allocations made must not grow
//! with `m`. A doc that claims it without a check here fails the build.

mod common;
mod counting;
use common::program;

/// The claims, keyed `file Type.fn` (or `file Type` for a type's own
/// doc), with a program that calls the claimed fn `m` times, `m` read.
const CHECKS: &[(&str, &str)] = &[
    (
        // std-api/R17: a compile-time `k`'s windows are views.
        "seq.cp windows",
        "let m = read<Int>()
let a = List.filled(m, 1)
var t = 0
for w in a.windows(3): t += w.len
print(t)",
    ),
    (
        "bag.cp SortedRange",
        "let m = read<Int>()
let s = SortedSet.of([1, 3, 7, 9])
var t = 0
for _ in 0..m:
    for x in s.range(2..=8): t += x
print(t)",
    ),
    (
        "bitset.cp Bitset.union",
        "domain D = 0..256
let m = read<Int>()
var b = Bitset<D>.single(D(1))
let o = Bitset<D>.of([D(1), D(2)])
for _ in 0..m: b.union(o)
print(b.len)",
    ),
    (
        "bitset.cp Bitset.intersect",
        "domain D = 0..256
let m = read<Int>()
var b = Bitset<D>.of([D(1), D(2)])
let o = Bitset<D>.single(D(2))
for _ in 0..m: b.intersect(o)
print(b.len)",
    ),
    (
        "bitset.cp Bitset.shift_or",
        "domain D = 0..256
let m = read<Int>()
var b = Bitset<D>.single(D(1))
for _ in 0..m: b.shift_or(1)
print(b.len)",
    ),
    (
        "bitset.cp DynBitset.shift_or",
        "use std.bitset.{DynBitset}
let m = read<Int>()
var d = DynBitset(256)
d.add(1)
for _ in 0..m: d.shift_or(1)
print(d.each().count())",
    ),
    (
        "deque.cp Deque.each",
        "let m = read<Int>()
let q = Deque.of([1, 2, 3, 4])
var t = 0
for _ in 0..m:
    for x in q.each(): t += x
print(t)",
    ),
    (
        "deque.cp Deque.each_rev",
        "let m = read<Int>()
let q = Deque.of([1, 2, 3, 4])
var t = 0
for _ in 0..m:
    for x in q.each_rev(): t += x
print(t)",
    ),
    (
        "dsu.cp Dsu.roots",
        "let m = read<Int>()
var d = Dsu<0..64>()
d.unite(3, 1)
var t = 0
for _ in 0..m:
    for _ in d.roots(): t += 1
print(t)",
    ),
    (
        "dsu.cp DsuOf.roots",
        "let m = read<Int>()
var d = DsuOf<0..64, Int>(init: v => Int(v), join: (x, y) => x + y)
d.unite(2, 0)
var t = 0
for _ in 0..m:
    for _ in d.roots(): t += 1
print(t)",
    ),
    (
        "dsu.cp DsuOf.values",
        "let m = read<Int>()
var d = DsuOf<0..64, Int>(init: v => Int(v), join: (x, y) => x + y)
d.unite(2, 0)
var t = 0
for _ in 0..m:
    for v in d.values: t += v
print(t)",
    ),
    (
        "dsu.cp PersistentDsu.unite",
        "use std.dsu.{DsuNode, PersistentDsu}
use std.persistent.{PArena}
domain V = 0..8
let m = read<Int>()
var ar = PArena<DsuNode<V>>()
var d = PersistentDsu<V>.new(mut ar)
d = d.unite(mut ar, 0, 1)
for _ in 0..m: d = d.unite(mut ar, 1, 0)
print(d.size(ar, 1))",
    ),
    (
        "heap.cp MeldHeaps",
        "use std.heap.{Meld, MeldHeaps}
let m = read<Int>()
var p = MeldHeaps<Int>()
var h = p.of([5, 9])
let e = Meld<Int>()
var t = 0
for i in 0..m:
    p.push(mut h, i)
    t += p.pop(mut h) ?? 0
    h = p.meld(h, e)
print(t)",
    ),
    (
        "math.cp Sieve.divisors",
        "let m = read<Int>()
let s = Sieve(1000)
var t = 0
for _ in 0..m:
    for x in s.divisors(720): t += x
print(t)",
    ),
    (
        "tree.cp Centroids.ancestors",
        "use std.tree.{Centroids}
domain V = 0..5
let m = read<Int>()
let adj: [V]List<V> = [[1, 2], [0, 3, 4], [0], [1], [1]]
let cd = Centroids(adj)
var t = 0
for _ in 0..m:
    for a in cd.ancestors(2): t += a.dist
print(t)",
    ),
];

/// Claims the VM breaks today, with why; the list only shrinks. A fix that
/// stops one allocating must take it off.
const ALLOCATING_TODAY: &[(&str, &str)] = &[
    (
        "seq.cp windows",
        "the VM boxes each slice view (`Rc<ListViewVal>`)",
    ),
    (
        "bag.cp SortedRange",
        "the VM boxes each cursor the walk steps to",
    ),
    (
        "heap.cp MeldHeaps",
        "the VM boxes each `MeldNode` and `Meld` struct value",
    ),
    (
        "math.cp Sieve.divisors",
        "std builds the factorization and exponents as lists",
    ),
    (
        "tree.cp Centroids.ancestors",
        "the VM boxes each yielded named tuple",
    ),
];

/// What a doc says when it promises no allocation.
const CLAIM: &[&str] = &[
    "no allocation",
    "without allocating",
    "allocates nothing",
    "zero allocation",
    "does not allocate",
];

/// The name an item line declares: `Bitset` in `impl<D: Domain> Bitset<D>:`,
/// `union` in `pub fn union(mut self, o: Bitset<D>):`.
fn declared(line: &str) -> Option<String> {
    let mut rest = line.trim();
    for word in ["@get ", "pub "] {
        rest = rest.strip_prefix(word).unwrap_or(rest).trim_start();
    }
    let (keyword, mut rest) = rest.split_once([' ', '<'])?;
    if !matches!(
        keyword,
        "fn" | "struct" | "enum" | "type" | "impl" | "trait"
    ) {
        return None;
    }
    if line.trim_start().starts_with("impl") && line.contains('<') && rest.contains('>') {
        let before_name = rest.find(|c: char| c.is_ascii_uppercase())?;
        rest = &rest[before_name..];
        if let Some((_, after)) = rest.split_once(" for ") {
            rest = after;
        }
    }
    let name: String = rest
        .trim_start()
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    (!name.is_empty()).then_some(name)
}

/// Every `file Type.fn` whose doc claims no allocation.
fn claims() -> Vec<String> {
    let std_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../std");
    let mut files: Vec<_> = std::fs::read_dir(&std_dir)
        .expect("std/ is next to the crates")
        .map(|e| e.expect("a std entry").path())
        .filter(|p| p.extension().is_some_and(|x| x == "cp"))
        .collect();
    files.sort();
    let mut found = Vec::new();
    for path in files {
        let file = path.file_name().unwrap().to_string_lossy().into_owned();
        let text = std::fs::read_to_string(&path).expect("a std file reads");
        let lines: Vec<&str> = text.lines().collect();
        let mut owner: Option<String> = None;
        for (i, line) in lines.iter().enumerate() {
            if !line.starts_with(' ') && !line.starts_with("///") {
                if let Some(name) = declared(line) {
                    owner = Some(name);
                }
            }
            let doc = line.trim_start();
            if !doc.starts_with("///") || !CLAIM.iter().any(|c| doc.contains(c)) {
                continue;
            }
            let item = lines[i..]
                .iter()
                .find(|l| !l.trim_start().starts_with("///"))
                .and_then(|l| declared(l).map(|name| (l.starts_with(' '), name)));
            match item {
                Some((true, name)) => {
                    found.push(format!("{file} {}.{name}", owner.as_deref().unwrap_or("?")))
                }
                Some((false, name)) => found.push(format!("{file} {name}")),
                None => found.push(format!("{file}:{} ?", i + 1)),
            }
        }
    }
    found.sort();
    found.dedup();
    found
}

/// How many allocations a run of `prog` on input `m` makes.
fn allocations(prog: &cpl_ir::Program, m: usize) -> usize {
    let mut out = Vec::with_capacity(64);
    let input = format!("{m}\n");
    let before = counting::usage().1;
    cpl_interp::run(prog, input.as_bytes(), &mut out).expect("the check runs");
    counting::usage().1 - before
}

#[test]
fn every_claim_of_no_allocation_has_a_check() {
    let checked: Vec<&str> = CHECKS.iter().map(|(key, _)| *key).collect();
    let unchecked: Vec<String> = claims()
        .into_iter()
        .filter(|claim| !checked.contains(&claim.as_str()))
        .collect();
    assert!(
        unchecked.is_empty(),
        "these docs say they allocate nothing, with no check in CHECKS: {unchecked:?}"
    );
}

#[test]
fn a_fn_that_claims_no_allocation_allocates_nothing_per_call() {
    const M: usize = 200;
    let mut grew = Vec::new();
    let mut fixed = Vec::new();
    for (key, source) in CHECKS {
        let prog = program(source);
        allocations(&prog, M);
        let few = allocations(&prog, M);
        let many = allocations(&prog, 4 * M);
        // The run's own setup (its thread, the input, a list sized by `m`)
        // may vary by a handful; a per-call allocation adds 3m.
        let allocates = many > few + 8;
        let known = ALLOCATING_TODAY.iter().any(|(k, _)| k == key);
        if allocates && !known {
            grew.push(format!(
                "{key}: {few} allocations at m = {M}, {many} at m = {}",
                4 * M
            ));
        } else if !allocates && known {
            fixed.push(*key);
        }
    }
    assert!(
        grew.is_empty(),
        "allocations grew with the calls:\n{}",
        grew.join("\n")
    );
    assert!(
        fixed.is_empty(),
        "these allocate nothing now; take them off ALLOCATING_TODAY: {fixed:?}"
    );
}
