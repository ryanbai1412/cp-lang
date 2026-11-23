//! End-to-end stress and contest-test coverage across 10 corpus problems of
//! different shapes: arrays, arrays+queries, strings (multitest), trees
//! (parent-array and weighted edge-list), graphs (weighted and functional),
//! 2D grids, constructive (`@check`), and interactive (`@interactor`).
//!
//! Each problem is checked twice under `cpl stress`:
//!   1. The correct solution must agree across generated cases.
//!   2. A planted bug must fail with a deterministic shrunk counterexample
//!      printed in input format, recording time to first counterexample.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn cpl_bin() -> &'static str {
    env!("CARGO_BIN_EXE_cpl")
}

/// Extracts a JSON string value of `key` from `json`, unescaped (top-level or nested under `counterexample`).
fn json_string_field(json: &str, key: &str) -> Option<String> {
    let root = cpl_json::parse(json).ok()?;
    root.get(key)
        .or_else(|| root.get("counterexample")?.get(key))
        .and_then(cpl_json::Value::as_str)
        .map(str::to_string)
}

#[test]
fn cpl_test_passes_on_corpus_in_and_ans_including_multitest() {
    let root = repo_root();
    let problems = [
        ("noi15p1", "corpus/problems/noi15p1/spec.cp"),
        ("ccc18s4", "corpus/problems/ccc18s4/spec.cp"),
        ("btoi20p3", "corpus/problems/btoi20p3/spec.cp"),
        ("ccc04s5", "corpus/problems/ccc04s5/spec.cp"),
        ("ceoi19pp1", "corpus/problems/ceoi19pp1/spec.cp"),
        ("dmopc20c4p4", "corpus/problems/dmopc20c4p4/spec.cp"),
        ("dmopc21c4p2", "corpus/problems/dmopc21c4p2/spec.cp"),
        ("etsr23r1p4", "corpus/problems/etsr23r1p4/spec.cp"),
        ("ioi01p1", "corpus/problems/ioi01p1/spec.cp"),
        ("noip14p6", "corpus/problems/noip14p6/spec.cp"),
        ("shoi16p3", "corpus/problems/shoi16p3/spec.cp"),
    ];
    for (id, rel_port) in problems {
        let port = root.join(rel_port);
        let tests_dir = root.join("corpus/problems").join(id).join("tests");
        let out = Command::new(cpl_bin())
            .arg("test")
            .arg(&port)
            .arg("--tests")
            .arg(&tests_dir)
            // The suite runs beside every other test on every core, so the judge's own
            // limit measures the load, not the solution; this test is about answers.
            .arg("--time-limit")
            .arg("30000")
            .output()
            .expect("cpl test runs");
        let stdout = String::from_utf8_lossy(&out.stdout);
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            out.status.success(),
            "cpl test failed on {id}:\n{stdout}\n{stderr}"
        );
        assert!(
            stdout.contains("8/8 passed"),
            "expected 8/8 passed on {id}:\n{stdout}\n{stderr}"
        );
    }

    // Auto-discovery of sibling `tests/*.in/.ans` with zero flags (multitest `noi15p1`).
    let scratch = Path::new(env!("CARGO_TARGET_TMPDIR")).join("corpus_sibling_tests");
    let _ = std::fs::remove_dir_all(&scratch);
    let tests_copy = scratch.join("tests");
    std::fs::create_dir_all(&tests_copy).unwrap();
    let sol = scratch.join("noi15p1.cp");
    std::fs::copy(root.join("corpus/problems/noi15p1/spec.cp"), &sol).unwrap();
    for entry in std::fs::read_dir(root.join("corpus/problems/noi15p1/tests")).unwrap() {
        let path = entry.unwrap().path();
        std::fs::copy(&path, tests_copy.join(path.file_name().unwrap())).unwrap();
    }
    let out = Command::new(cpl_bin())
        .arg("test")
        .arg(&sol)
        .output()
        .expect("cpl test runs");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "{stdout}\n{stderr}");
    assert!(stdout.contains("8/8 passed"), "{stdout}\n{stderr}");
}

struct StressCase {
    id: &'static str,
    shape: &'static str,
    good: &'static str,
    buggy: &'static str,
    size: &'static str,
    good_cases: &'static str,
}

// 1. sac22cc2p5: Arrays + range-sum/point-update queries, `@alt(program)`, typed `read()`.
const SAC22CC2P5_GOOD: &str = r#"
let n: 1..=1_000_000 = read()
let q: 1..=1_000_000 = read()
domain Pos = 1..=n

var cur: [Pos]Int = 0
var fen = [Fenwick<Pos, Int>(), Fenwick<Pos, Int>()]

for i in Pos:
    let x: -100..=100 = read()
    cur[i] = x
    fen[i.ord % 2].add(i, x)

for _ in 0..q:
    let ty: 1..=2 = read()
    if ty == 1:
        let i: Pos = read()
        let x: -100..=100 = read()
        fen[i.ord % 2].add(i, x - cur[i])
        cur[i] = x
    else:
        let l: 1..=n = read()
        let r: l..=n = read()
        print(fen[Pos(l).ord % 2].fold(Pos(l)..=Pos(r)))

@alt(program)
fn brute():
    let n: 1..=1_000_000 = read()
    let q: 1..=1_000_000 = read()
    domain Pos = 1..=n
    var a: [Pos]Int = read()
    for _ in 0..q:
        let ty: 1..=2 = read()
        if ty == 1:
            let i: Pos = read()
            let x: Int = read()
            a[i] = x
        else:
            let l: 1..=n = read()
            let r: l..=n = read()
            var s = 0
            for i in Pos(l)..=Pos(r):
                if i.ord % 2 == Pos(l).ord % 2:
                    s += a[i]
            print(s)
"#;

const SAC22CC2P5_BUGGY: &str = r#"
let n: 1..=1_000_000 = read()
let q: 1..=1_000_000 = read()
domain Pos = 1..=n

var cur: [Pos]Int = 0
var fen = [Fenwick<Pos, Int>(), Fenwick<Pos, Int>()]

for i in Pos:
    let x: -100..=100 = read()
    cur[i] = x
    fen[i.ord % 2].add(i, x)

for _ in 0..q:
    let ty: 1..=2 = read()
    if ty == 1:
        let i: Pos = read()
        let x: -100..=100 = read()
        fen[i.ord % 2].add(i, x - cur[i])
        cur[i] = x
    else:
        let l: 1..=n = read()
        let r: l..=n = read()
        print(fen[Pos(r).ord % 2].fold(Pos(l)..=Pos(r)))

@alt(program)
fn brute():
    let n: 1..=1_000_000 = read()
    let q: 1..=1_000_000 = read()
    domain Pos = 1..=n
    var a: [Pos]Int = read()
    for _ in 0..q:
        let ty: 1..=2 = read()
        if ty == 1:
            let i: Pos = read()
            let x: Int = read()
            a[i] = x
        else:
            let l: 1..=n = read()
            let r: l..=n = read()
            var s = 0
            for i in Pos(l)..=Pos(r):
                if i.ord % 2 == Pos(l).ord % 2:
                    s += a[i]
            print(s)
"#;

// 2. dmopc20c4p4: Arrays (greedy stack), part-level `@alt(solve)`, user `@gen`.
const DMOPC20C4P4_GOOD: &str = r#"
fn solve(n: Int, a: List<Int>) -> List<Int>:
    var have = 0
    var ans = 0
    var spare: List<Int> = []
    var out: List<Int> = []
    for i in 0..n:
        let x = a[i]
        while have < x:
            let j = spare.last!
            spare.pop()
            ans += i - j
            have += 1
        if x == have:
            have += 1
        else:
            ans += have - x - 1
            spare.push(i)
        out.push(ans)
    out

@alt(solve)
fn solve_slow(n: Int, a: List<Int>) -> List<Int>:
    var is_spare: List<Bool> = [false for _ in 0..n]
    var have = 0
    var ops = 0
    var out: List<Int> = []
    for i in 0..n:
        let x = a[i]
        while have < x:
            var j = i - 1
            while not is_spare[j]:
                j -= 1
            is_spare[j] = false
            ops += i - j
            have += 1
        if x == have:
            have += 1
        else:
            ops += have - x - 1
            is_spare[i] = true
        out.push(ops)
    out

let n: 1..=100 = read()
let a: List<Int> = [read() for _ in 0..n]
for x in solve(n, a): print(x)

@gen
fn make_input(size: Int):
    let n = rand(1, size)
    print(n)
    var xs: List<Int> = []
    for i in 0..n:
        xs.push(rand(0, i))
    print(xs)
"#;

const DMOPC20C4P4_BUGGY: &str = r#"
fn solve(n: Int, a: List<Int>) -> List<Int>:
    var have = 0
    var ans = 0
    var spare: List<Int> = []
    var out: List<Int> = []
    for i in 0..n:
        let x = a[i]
        while have < x:
            let j = spare.last!
            spare.pop()
            ans += i - j
            have += 1
        if x == have:
            have += 1
        else:
            ans += have - x
            spare.push(i)
        out.push(ans)
    out

@alt(solve)
fn solve_slow(n: Int, a: List<Int>) -> List<Int>:
    var have = 0
    var ans = 0
    var spare: List<Int> = []
    var out: List<Int> = []
    for i in 0..n:
        let x = a[i]
        while have < x:
            let j = spare.last!
            spare.pop()
            ans += i - j
            have += 1
        if x == have:
            have += 1
        else:
            ans += have - x - 1
            spare.push(i)
        out.push(ans)
    out

let n: 1..=100 = read()
let a: List<Int> = [read() for _ in 0..n]
for x in solve(n, a): print(x)

@gen
fn make_input(size: Int):
    let n = rand(1, size)
    print(n)
    var xs: List<Int> = []
    for i in 0..n:
        xs.push(rand(0, i))
    print(xs)
"#;

// 3. ccc03s4: Strings + `@!multitest`, `@alt(program)`, typed `Str` gen + multitest shrinking.
const CCC03S4_GOOD: &str = r#"
use std.strings.{SuffixArray}

@!multitest
let s: Str = read()
print(1 + SuffixArray.new(s).distinct)

@alt(program)
fn brute():
    var seen = Set<Str>()
    for i in 0..s.len:
        for j in i + 1..=s.len:
            seen.insert(s[i..j].to_str())
    print(1 + seen.len)
"#;

const CCC03S4_BUGGY: &str = r#"
use std.strings.{SuffixArray}

@!multitest
let s: Str = read()
print(1 + s.len * (s.len + 1) / 2)

@alt(program)
fn brute():
    var seen = Set<Str>()
    for i in 0..s.len:
        for j in i + 1..=s.len:
            seen.insert(s[i..j].to_str())
    print(1 + seen.len)
"#;

// 4. btoi17p6: Trees (parent array with `read(gen: random_parents)`), `@alt(program)`.
const BTOI17P6_GOOD: &str = r#"
let n: 1..=20 = read()
let d: 1..=20 = read()
domain V = 0..n
domain Kid = 1..n

fn random_parents() -> [Kid]V:
    [Kid] k => V(rand(0, k.label - 1))

let parent: [Kid]V = read(gen: random_parents)

type Pick = (count: Int, near: Int)

fn join(a: Pick, b: Pick) -> Pick:
    if a.near + b.near >= d: (a.count + b.count, min(a.near, b.near))
    else: (a.count + b.count - 1, max(a.near, b.near))

var pick: [V]Pick = (1, 0)
for k in Kid.rev:
    let below = pick[V(k.label)]
    pick[parent[k]] = join(pick[parent[k]], (below.count, below.near + 1))
print(pick[V.first!].count)

@alt(program)
fn brute():
    var dist: [0..n, 0..n]Int = 1000
    for i in 0..n: dist[i, i] = 0
    for k in Kid:
        let u = k.label
        let p = parent[k].label
        dist[u, p] = 1
        dist[p, u] = 1
    for m in 0..n:
        for i in 0..n:
            for j in 0..n:
                dist[i, j] min= dist[i, m] + dist[m, j]
    var best = 0
    for mask in 0..(1 << n):
        var ok = true
        var cnt = 0
        for i in 0..n:
            if ((mask >> i) & 1) == 1:
                cnt += 1
                for j in i + 1..n:
                    if ((mask >> j) & 1) == 1 and dist[i, j] < d:
                        ok = false
        if ok:
            best max= cnt
    print(best)
"#;

const BTOI17P6_BUGGY: &str = r#"
let n: 1..=20 = read()
let d: 1..=20 = read()
domain V = 0..n
domain Kid = 1..n

fn random_parents() -> [Kid]V:
    [Kid] k => V(rand(0, k.label - 1))

let parent: [Kid]V = read(gen: random_parents)

type Pick = (count: Int, near: Int)

fn join(a: Pick, b: Pick) -> Pick:
    if a.near + b.near > d: (a.count + b.count, min(a.near, b.near))
    else: (a.count + b.count - 1, max(a.near, b.near))

var pick: [V]Pick = (1, 0)
for k in Kid.rev:
    let below = pick[V(k.label)]
    pick[parent[k]] = join(pick[parent[k]], (below.count, below.near + 1))
print(pick[V.first!].count)

@alt(program)
fn brute():
    var dist: [0..n, 0..n]Int = 1000
    for i in 0..n: dist[i, i] = 0
    for k in Kid:
        let u = k.label
        let p = parent[k].label
        dist[u, p] = 1
        dist[p, u] = 1
    for m in 0..n:
        for i in 0..n:
            for j in 0..n:
                dist[i, j] min= dist[i, m] + dist[m, j]
    var best = 0
    for mask in 0..(1 << n):
        var ok = true
        var cnt = 0
        for i in 0..n:
            if ((mask >> i) & 1) == 1:
                cnt += 1
                for j in i + 1..n:
                    if ((mask >> j) & 1) == 1 and dist[i, j] < d:
                        ok = false
        if ok:
            best max= cnt
    print(best)
"#;

// 5. shoi16p1: Trees (weighted edge-list DP), user `@gen` + choice shrinking, `@alt(program)`.
const SHOI16P1_GOOD: &str = r#"
let n: 1..=15 = read()
let k: 0..=15 = read()
domain V = 1..=n
let cost: [V]Int = read()
let m: 0..=n = read()
var marked: [V]Bool = false
for _ in 0..m:
    let u: V = read()
    marked[u] = true
var adj: [V]List<V> = []
for _ in 0..n - 1:
    let u: V = read()
    let v: V = read()
    adj[u].push(v)
    adj[v].push(u)

var kids: [V]List<V> = []
var seen: [V]Bool = false
var order: List<V> = []
order.push(V.first!)
seen[V.first!] = true
var head = 0
while head < order.len:
    let u = order[head]
    head += 1
    for v in adj[u]:
        if not seen[v]:
            seen[v] = true
            kids[u].push(v)
            order.push(v)

var need: [V, 0..=2 * k]Int = 0
for at in (0..n).rev:
    let u = order[at]
    for d in (-k..=k).rev:
        var best = 1e18
        if d < k: best = need[u, d + 1 + k]
        if d == k:
            var here = cost[u]
            for v in kids[u]: here += need[v, 0]
            best min= here
        elif d >= 0:
            if kids[u].len > 0:
                var rest = 0
                var extra = 1e18
                for v in kids[u]:
                    rest += need[v, k - d]
                    extra min= need[v, d + 1 + k] - need[v, k - d]
                best min= rest + extra
        else:
            var below = 0
            for v in kids[u]: below += need[v, d + 1 + k]
            best min= below
        if d == 0 and not marked[u]:
            var below = 0
            for v in kids[u]: below += need[v, k]
            best min= below
        need[u, d + k] = best
print(need[V.first!, k])

@alt(program)
fn brute():
    var dist: [0..n, 0..n]Int = 1000
    for i in 0..n: dist[i, i] = 0
    for u in V:
        for v in adj[u]:
            dist[u.ord, v.ord] = 1
    for mid in 0..n:
        for i in 0..n:
            for j in 0..n:
                dist[i, j] min= dist[i, mid] + dist[mid, j]
    var best = 1e18
    for mask in 0..(1 << n):
        var total = 0
        for i in 0..n:
            if ((mask >> i) & 1) == 1:
                total += cost[V(i + 1)]
        var ok = true
        for u in V:
            if marked[u]:
                var covered = false
                for i in 0..n:
                    if ((mask >> i) & 1) == 1 and dist[u.ord, i] <= k:
                        covered = true
                if not covered:
                    ok = false
        if ok:
            best min= total
    print(best)

@gen
fn make_tree(size: Int):
    let n = rand(1, size)
    let k = rand(0, n)
    print(n, k)
    print([rand(1, 20) for _ in 0..n])
    var marks: List<Int> = []
    for u in 1..=n:
        if rand(0, 1) == 1:
            marks.push(u)
    print(marks.len)
    if marks.len > 0:
        print(marks)
    for v in 2..=n:
        print(rand(1, v - 1), v)
"#;

const SHOI16P1_BUGGY: &str = r#"
let n: 1..=15 = read()
let k: 0..=15 = read()
domain V = 1..=n
let cost: [V]Int = read()
let m: 0..=n = read()
var marked: [V]Bool = false
for _ in 0..m:
    let u: V = read()
    marked[u] = true
var adj: [V]List<V> = []
for _ in 0..n - 1:
    let u: V = read()
    let v: V = read()
    adj[u].push(v)
    adj[v].push(u)

var kids: [V]List<V> = []
var seen: [V]Bool = false
var order: List<V> = []
order.push(V.first!)
seen[V.first!] = true
var head = 0
while head < order.len:
    let u = order[head]
    head += 1
    for v in adj[u]:
        if not seen[v]:
            seen[v] = true
            kids[u].push(v)
            order.push(v)

var need: [V, 0..=2 * k]Int = 0
for at in (0..n).rev:
    let u = order[at]
    for d in (-k..=k).rev:
        var best = 1e18
        if d < k: best = need[u, d + 1 + k]
        if d == k:
            var here = cost[u]
            for v in kids[u]: here += need[v, 0]
            best min= here
        elif d >= 0:
            if kids[u].len > 0:
                var rest = 0
                var extra = 1e18
                for v in kids[u]:
                    rest += need[v, k - d]
                    extra min= need[v, d + 1 + k] - need[v, k - d]
                best min= rest + extra
        else:
            var below = 0
            for v in kids[u]: below += need[v, d + 1 + k]
            best min= below
        need[u, d + k] = best
print(need[V.first!, k])

@alt(program)
fn brute():
    var dist: [0..n, 0..n]Int = 1000
    for i in 0..n: dist[i, i] = 0
    for u in V:
        for v in adj[u]:
            dist[u.ord, v.ord] = 1
    for mid in 0..n:
        for i in 0..n:
            for j in 0..n:
                dist[i, j] min= dist[i, mid] + dist[mid, j]
    var best = 1e18
    for mask in 0..(1 << n):
        var total = 0
        for i in 0..n:
            if ((mask >> i) & 1) == 1:
                total += cost[V(i + 1)]
        var ok = true
        for u in V:
            if marked[u]:
                var covered = false
                for i in 0..n:
                    if ((mask >> i) & 1) == 1 and dist[u.ord, i] <= k:
                        covered = true
                if not covered:
                    ok = false
        if ok:
            best min= total
    print(best)

@gen
fn make_tree(size: Int):
    let n = rand(1, size)
    let k = rand(0, n)
    print(n, k)
    print([rand(1, 20) for _ in 0..n])
    var marks: List<Int> = []
    for u in 1..=n:
        if rand(0, 1) == 1:
            marks.push(u)
    print(marks.len)
    if marks.len > 0:
        print(marks)
    for v in 2..=n:
        print(rand(1, v - 1), v)
"#;

// 6. ucrpc21h: Undirected weighted graph (Dijkstra + 3D DP), user `@gen`, `@alt(program)`.
const UCRPC21H_GOOD: &str = r#"
const LIMIT = 30
let n: 1..=10 = read()
let m: 0..=40 = read()
domain V = 0..n
var adj: [V]List<(V, Int)> = []
var too_long = false
for _ in 0..m:
    let u: V = read()
    let v: V = read()
    let w: Int = read()
    if w > LIMIT: too_long = true
    adj[u].push((v, w))
    adj[v].push((u, w))

var dist: [V]Int = 1e18
var pq = MinHeap<(Int, V)>()
dist[V.first!] = 0
pq.push((0, V.first!))
while pq.len > 0:
    let (d, u) = pq.pop()!
    if d == dist[u]:
        for (v, w) in adj[u]:
            if d + w < dist[v]:
                dist[v] = d + w
                pq.push((d + w, v))

var total = 0
var far = 0
for v in V:
    total += dist[v]
    far max= dist[v]

if too_long or far > LIMIT or total > 4 * LIMIT:
    print("Impossible!")
else:
    let cap = min(LIMIT, total)
    var split: [0..=cap, 0..=cap, 0..=cap]Bool = false
    split[0, 0, 0] = true
    for v in V:
        let x = dist[v]
        if x > 0:
            for a in (0..=cap).rev:
                for b in (0..=cap).rev:
                    for c in (0..=cap).rev:
                        if not split[a, b, c]:
                            if (a >= x and split[a - x, b, c]) or (b >= x and split[a, b - x, c]) or (c >= x and split[a, b, c - x]):
                                split[a, b, c] = true
    var best = 1e18
    for a in 0..=cap:
        for b in 0..=cap:
            for c in 0..=cap:
                if split[a, b, c]: best min= max(max(a, b), max(c, total - a - b - c))
    if best <= LIMIT: print(2 * best)
    else: print("Impossible!")

@alt(program)
fn brute():
    var dmat: [0..n, 0..n]Int = 1e9
    for i in 0..n: dmat[i, i] = 0
    for u in V:
        for (v, w) in adj[u]:
            dmat[u.ord, v.ord] min= w
    for k in 0..n:
        for i in 0..n:
            for j in 0..n:
                dmat[i, j] min= dmat[i, k] + dmat[k, j]
    var far = 0
    for i in 0..n: far max= dmat[0, i]
    if too_long or far > LIMIT:
        print("Impossible!")
        return
    var best = 1e9
    fn go(i: Int, s0: Int, s1: Int, s2: Int, s3: Int):
        if i == n:
            best min= max(max(s0, s1), max(s2, s3))
            return
        let x = dmat[0, i]
        if s0 + x <= LIMIT: go(i + 1, s0 + x, s1, s2, s3)
        if s1 + x <= LIMIT: go(i + 1, s0, s1 + x, s2, s3)
        if s2 + x <= LIMIT: go(i + 1, s0, s1, s2 + x, s3)
        if s3 + x <= LIMIT: go(i + 1, s0, s1, s2, s3 + x)
    go(1, 0, 0, 0, 0)
    if best <= LIMIT: print(2 * best)
    else: print("Impossible!")

@gen
fn make_graph(size: Int):
    let n = rand(1, size)
    let extra = rand(0, n)
    print(n, (n - 1) + extra)
    for v in 1..n:
        print(rand(0, v - 1), v, rand(1, 15))
    for _ in 0..extra:
        print(rand(0, n - 1), rand(0, n - 1), rand(1, 15))
"#;

const UCRPC21H_BUGGY: &str = r#"
const LIMIT = 30
let n: 1..=10 = read()
let m: 0..=40 = read()
domain V = 0..n
var adj: [V]List<(V, Int)> = []
var too_long = false
for _ in 0..m:
    let u: V = read()
    let v: V = read()
    let w: Int = read()
    if w > LIMIT: too_long = true
    adj[u].push((v, w))
    adj[v].push((u, w))

var dist: [V]Int = 1e18
var pq = MinHeap<(Int, V)>()
dist[V.first!] = 0
pq.push((0, V.first!))
while pq.len > 0:
    let (d, u) = pq.pop()!
    if d == dist[u]:
        for (v, w) in adj[u]:
            if d + w < dist[v]:
                dist[v] = d + w
                pq.push((d + w, v))

var total = 0
var far = 0
for v in V:
    total += dist[v]
    far max= dist[v]

if too_long or far > LIMIT or total > 4 * LIMIT:
    print("Impossible!")
else:
    let cap = min(LIMIT, total)
    var split: [0..=cap, 0..=cap, 0..=cap]Bool = false
    split[0, 0, 0] = true
    for v in V:
        let x = dist[v]
        if x > 0:
            for a in (0..=cap).rev:
                for b in (0..=cap).rev:
                    for c in (0..=cap).rev:
                        if not split[a, b, c]:
                            if (a >= x and split[a - x, b, c]) or (b >= x and split[a, b - x, c]) or (c >= x and split[a, b, c - x]):
                                split[a, b, c] = true
    var best = 1e18
    for a in 0..=cap:
        for b in 0..=cap:
            for c in 0..=cap:
                if split[a, b, c]: best min= max(max(a, b), total - a - b - c)
    if best <= LIMIT: print(2 * best)
    else: print("Impossible!")

@alt(program)
fn brute():
    var dmat: [0..n, 0..n]Int = 1e9
    for i in 0..n: dmat[i, i] = 0
    for u in V:
        for (v, w) in adj[u]:
            dmat[u.ord, v.ord] min= w
    for k in 0..n:
        for i in 0..n:
            for j in 0..n:
                dmat[i, j] min= dmat[i, k] + dmat[k, j]
    var far = 0
    for i in 0..n: far max= dmat[0, i]
    if too_long or far > LIMIT:
        print("Impossible!")
        return
    var best = 1e9
    fn go(i: Int, s0: Int, s1: Int, s2: Int, s3: Int):
        if i == n:
            best min= max(max(s0, s1), max(s2, s3))
            return
        let x = dmat[0, i]
        if s0 + x <= LIMIT: go(i + 1, s0 + x, s1, s2, s3)
        if s1 + x <= LIMIT: go(i + 1, s0, s1 + x, s2, s3)
        if s2 + x <= LIMIT: go(i + 1, s0, s1, s2 + x, s3)
        if s3 + x <= LIMIT: go(i + 1, s0, s1, s2, s3 + x)
    go(1, 0, 0, 0, 0)
    if best <= LIMIT: print(2 * best)
    else: print("Impossible!")

@gen
fn make_graph(size: Int):
    let n = rand(1, size)
    let extra = rand(0, n)
    print(n, (n - 1) + extra)
    for v in 1..n:
        print(rand(0, v - 1), v, rand(1, 15))
    for _ in 0..extra:
        print(rand(0, n - 1), rand(0, n - 1), rand(1, 15))
"#;

// 7. yac7p3: Functional graph `par: [V]V`, typed `read()`, `@alt(program)`.
const YAC7P3_GOOD: &str = r#"
let n: 1..=20 = read()
domain V = 1..=n
var par: [V]V = read()

var rev: [V]List<V> = []
var vis: [V]Int = 0
var on_cyc: [V]Bool = false
var st: List<V> = []
var cyc_nodes: List<V> = []
var cyc_starts: List<Int> = []

fn dfs_cyc(u: V):
    if vis[u] != 0:
        if vis[u] == 1:
            var pos = 0
            while pos < st.len and st[pos] != u:
                pos += 1
            cyc_starts.push(cyc_nodes.len)
            for i in pos..st.len:
                let v = st[i]
                on_cyc[v] = true
                cyc_nodes.push(v)
        return
    vis[u] = 1
    st.push(u)
    for i in 0..rev[u].len:
        dfs_cyc(rev[u][i])
    vis[u] = 2
    st.pop()

fn run_cycles():
    for v in V:
        rev[v] = []
        vis[v] = 0
        on_cyc[v] = false
    cyc_nodes = []
    cyc_starts = []
    for v in V:
        rev[par[v]].push(v)
    for v in V:
        dfs_cyc(v)
    cyc_starts.push(cyc_nodes.len)

run_cycles()

var depth: [V]Int = -1
var anchor: [V]V = V.first!

fn dfs_depth(u: V, root: V, d: Int):
    depth[u] = d
    anchor[u] = root
    for i in 0..rev[u].len:
        let v = rev[u][i]
        if not on_cyc[v]:
            dfs_depth(v, root, d + 1)

for v in V:
    if on_cyc[v]:
        dfs_depth(v, v, -1)

var best_d = 0
var u_nv = V.first!
for v in V:
    if depth[v] > best_d or (depth[v] == best_d and v.idx > u_nv.idx):
        best_d = depth[v]
        u_nv = v

let target_root = anchor[u_nv]
var u_ind = V.first!
for v in V:
    if on_cyc[v] and par[v] == target_root:
        u_ind = v

par[u_ind] = u_nv
run_cycles()
print(cyc_nodes.len)

@alt(program)
fn brute():
    var best = 0
    for u in V:
        let old = par[u]
        for w in V:
            par[u] = w
            var cyc = 0
            for s in V:
                var cur = s
                for _ in 0..n:
                    cur = par[cur]
                var in_c = false
                var x = cur
                for _ in 0..n:
                    if x == s:
                        in_c = true
                    x = par[x]
                if in_c:
                    cyc += 1
            best max= cyc
        par[u] = old
    print(best)
"#;

const YAC7P3_BUGGY: &str = r#"
let n: 1..=20 = read()
domain V = 1..=n
var par: [V]V = read()

var rev: [V]List<V> = []
var vis: [V]Int = 0
var on_cyc: [V]Bool = false
var st: List<V> = []
var cyc_nodes: List<V> = []
var cyc_starts: List<Int> = []

fn dfs_cyc(u: V):
    if vis[u] != 0:
        if vis[u] == 1:
            var pos = 0
            while pos < st.len and st[pos] != u:
                pos += 1
            cyc_starts.push(cyc_nodes.len)
            for i in pos..st.len:
                let v = st[i]
                on_cyc[v] = true
                cyc_nodes.push(v)
        return
    vis[u] = 1
    st.push(u)
    for i in 0..rev[u].len:
        dfs_cyc(rev[u][i])
    vis[u] = 2
    st.pop()

fn run_cycles():
    for v in V:
        rev[v] = []
        vis[v] = 0
        on_cyc[v] = false
    cyc_nodes = []
    cyc_starts = []
    for v in V:
        rev[par[v]].push(v)
    for v in V:
        dfs_cyc(v)
    cyc_starts.push(cyc_nodes.len)

run_cycles()

var depth: [V]Int = -1
var anchor: [V]V = V.first!

fn dfs_depth(u: V, root: V, d: Int):
    depth[u] = d
    anchor[u] = root
    for i in 0..rev[u].len:
        let v = rev[u][i]
        if not on_cyc[v]:
            dfs_depth(v, root, d + 1)

for v in V:
    if on_cyc[v]:
        dfs_depth(v, v, -1)

var best_d = 1
var u_nv = V.first!
for v in V:
    if depth[v] > best_d or (depth[v] == best_d and v.idx > u_nv.idx):
        best_d = depth[v]
        u_nv = v

let target_root = anchor[u_nv]
var u_ind = V.first!
for v in V:
    if on_cyc[v] and par[v] == target_root:
        u_ind = v

par[u_ind] = u_nv
run_cycles()
print(cyc_nodes.len)

@alt(program)
fn brute():
    var best = 0
    for u in V:
        let old = par[u]
        for w in V:
            par[u] = w
            var cyc = 0
            for s in V:
                var cur = s
                for _ in 0..n:
                    cur = par[cur]
                var in_c = false
                var x = cur
                for _ in 0..n:
                    if x == s:
                        in_c = true
                    x = par[x]
                if in_c:
                    cyc += 1
            best max= cyc
        par[u] = old
    print(best)
"#;

// 8. ccc04s5: 2D Grid DP, user `@gen` + choice shrinking, `@alt(program)`.
const CCC04S5_GOOD: &str = r#"
let rows: 1..=10 = read()
let cols: 1..=10 = read()
domain R = 0..rows
domain C = 0..cols
let grid: [R, C]Char = read()
let neg_inf = -1_000_000_000

var dp: [R]Int = neg_inf
dp[R(rows - 1)] = 0

var c = cols - 1
while c >= 0:
    var nxt: [R]Int = neg_inf
    for start in 0..rows:
        var gain = 0
        for r in start..rows:
            let ch = grid[R(r), C(c)]
            if ch == '*': break
            if ch >= '0' and ch <= '9': gain += ch - '0'
            if dp[R(r)] > neg_inf: nxt[R(start)] max= gain + dp[R(r)]
        gain = 0
        var r = start
        while r >= 0:
            let ch = grid[R(r), C(c)]
            if ch == '*': break
            if ch >= '0' and ch <= '9': gain += ch - '0'
            if dp[R(r)] > neg_inf: nxt[R(start)] max= gain + dp[R(r)]
            r -= 1
    dp = nxt
    c -= 1
print(dp[R(rows - 1)])

@alt(program)
fn brute():
    // Forward DP by column from left to right.
    let neg_inf = -1_000_000_000
    var cur: [R]Int = neg_inf
    cur[R(rows - 1)] = 0
    for col in 0..cols:
        var nxt: [R]Int = neg_inf
        for start in 0..rows:
            if cur[R(start)] > neg_inf:
                var gain = 0
                for r in start..rows:
                    let ch = grid[R(r), C(col)]
                    if ch == '*': break
                    if ch >= '0' and ch <= '9': gain += ch - '0'
                    nxt[R(r)] max= cur[R(start)] + gain
                gain = 0
                var r = start
                while r >= 0:
                    let ch = grid[R(r), C(col)]
                    if ch == '*': break
                    if ch >= '0' and ch <= '9': gain += ch - '0'
                    nxt[R(r)] max= cur[R(start)] + gain
                    r -= 1
        cur = nxt
    print(cur[R(rows - 1)])

@gen
fn make_grid(size: Int):
    let rows = rand(2, size)
    let cols = rand(2, size)
    print(rows, cols)
    for r in 0..rows:
        var line = ""
        for c in 0..cols:
            if r == rows - 1 and (c == 0 or c == cols - 1):
                line += "."
            else:
                let k = rand(0, 4)
                if k == 0: line += "*"
                elif k == 1: line += "."
                else: line += Str(rand(1, 9))
        print(line)
"#;

const CCC04S5_BUGGY: &str = r#"
let rows: 1..=10 = read()
let cols: 1..=10 = read()
domain R = 0..rows
domain C = 0..cols
let grid: [R, C]Char = read()
let neg_inf = -1_000_000_000

var dp: [R]Int = neg_inf
dp[R(rows - 1)] = 0

var c = cols - 1
while c >= 0:
    var nxt: [R]Int = neg_inf
    for start in 0..rows:
        var gain = 0
        for r in start..rows:
            let ch = grid[R(r), C(c)]
            if ch == '*': break
            if ch >= '0' and ch <= '9': gain += ch - '0'
            if dp[R(r)] > neg_inf: nxt[R(start)] max= gain + dp[R(r)]
    dp = nxt
    c -= 1
print(dp[R(rows - 1)])

@alt(program)
fn brute():
    let neg_inf = -1_000_000_000
    var cur: [R]Int = neg_inf
    cur[R(rows - 1)] = 0
    for col in 0..cols:
        var nxt: [R]Int = neg_inf
        for start in 0..rows:
            if cur[R(start)] > neg_inf:
                var gain = 0
                for r in start..rows:
                    let ch = grid[R(r), C(col)]
                    if ch == '*': break
                    if ch >= '0' and ch <= '9': gain += ch - '0'
                    nxt[R(r)] max= cur[R(start)] + gain
                gain = 0
                var r = start
                while r >= 0:
                    let ch = grid[R(r), C(col)]
                    if ch == '*': break
                    if ch >= '0' and ch <= '9': gain += ch - '0'
                    nxt[R(r)] max= cur[R(start)] + gain
                    r -= 1
        cur = nxt
    print(cur[R(rows - 1)])

@gen
fn make_grid(size: Int):
    let rows = rand(2, size)
    let cols = rand(2, size)
    print(rows, cols)
    for r in 0..rows:
        var line = ""
        for c in 0..cols:
            if r == rows - 1 and (c == 0 or c == cols - 1):
                line += "."
            else:
                let k = rand(0, 4)
                if k == 0: line += "*"
                elif k == 1: line += "."
                else: line += Str(rand(1, 9))
        print(line)
"#;

// 9. aac5p4: Constructive math judged by in-file `@check` (custom checker!), typed `read()`.
const AAC5P4_GOOD: &str = r#"
let n: 1..=1_000_000 = read()
let x: 0..=1_000_000_000 = read()

if n == 1:
    print(0)
else:
    var ans = n / 2
    if n % 2 == 1:
        let h = (n - 3) / 2
        let flip = h % 4 == 0 or h % 4 == 3
        if flip != (x % 2 == 1):
            ans += 1
    print(ans)
    for i in 1..=ans:
        print(i)

@check
fn ok():
    let k: Int = read_output()
    var want = 0
    if n > 1:
        want = n / 2
        if n % 2 == 1:
            let h = (n - 3) / 2
            let flip = h % 4 == 0 or h % 4 == 3
            if flip != (x % 2 == 1):
                want += 1
    assert k == want
    var seen = Set<Int>()
    for _ in 0..k:
        let v: Int = read_output()
        assert v >= 1 and v <= n
        assert not seen.contains(v)
        seen.insert(v)
"#;

const AAC5P4_BUGGY: &str = r#"
let n: 1..=1_000_000 = read()
let x: 0..=1_000_000_000 = read()

if n == 1:
    print(0)
else:
    let ans = n / 2
    print(ans)
    for i in 1..=ans:
        print(i)

@check
fn ok():
    let k: Int = read_output()
    var want = 0
    if n > 1:
        want = n / 2
        if n % 2 == 1:
            let h = (n - 3) / 2
            let flip = h % 4 == 0 or h % 4 == 3
            if flip != (x % 2 == 1):
                want += 1
    assert k == want
    var seen = Set<Int>()
    for _ in 0..k:
        let v: Int = read_output()
        assert v >= 1 and v <= n
        assert not seen.contains(v)
        seen.insert(v)
"#;

// 10. cf2135b: Interactive + `@!multitest`, in-file `@interactor`, typed hidden input + shrinking.
const CF2135B_GOOD: &str = r#"
@!interactive

fn ask_move(d: Char, dist: Int) -> Int:
    print("?", d, dist)
    read()

@!multitest
let n: 1..=100 = read()
let anchors: [0..n](x: -100..=100, y: -100..=100) = read()
let step = 1_000_000_000

for d in "UUR": ask_move(d, step)
let d_ur = ask_move('R', step)
for _ in 0..3: ask_move('L', step)
let d_ul = ask_move('L', step)

let ur_pos = [p.x + p.y for p in anchors].max()! + d_ur - 4 * step
let ul_pos = [-p.x + p.y for p in anchors].max()! + d_ul - 4 * step
print("!", (ur_pos - ul_pos) / 2, (ur_pos + ul_pos) / 2)

@interactor
fn judge(mut sol: Interactor):
    let n: 1..=8 = read()
    let x0: -100..=100 = read()
    let y0: -100..=100 = read()
    let anchors: [0..n](x: -100..=100, y: -100..=100) = read()
    sol.send(n)
    for p in anchors:
        sol.send(p.x, p.y)
    var cx = x0
    var cy = y0
    var queries = 0
    while true:
        let kind: Str = sol.recv()
        if kind == "!":
            let gx: Int = sol.recv()
            let gy: Int = sol.recv()
            assert gx == x0 and gy == y0
            return
        assert kind == "?"
        queries += 1
        assert queries <= 10
        let dir: Str = sol.recv()
        let k: Int = sol.recv()
        if dir == "U": cy += k
        elif dir == "D": cy -= k
        elif dir == "R": cx += k
        elif dir == "L": cx -= k
        else: assert false
        var best = 1e18
        for p in anchors:
            best min= abs(cx - p.x) + abs(cy - p.y)
        sol.send(best)
"#;

const CF2135B_BUGGY: &str = r#"
@!interactive

fn ask_move(d: Char, dist: Int) -> Int:
    print("?", d, dist)
    read()

@!multitest
let n: 1..=100 = read()
let anchors: [0..n](x: -100..=100, y: -100..=100) = read()
let step = 1_000_000_000

for d in "UUR": ask_move(d, step)
let d_ur = ask_move('R', step)
for _ in 0..3: ask_move('L', step)
let d_ul = ask_move('L', step)

let ur_pos = [p.x + p.y for p in anchors].max()! + d_ur - 3 * step
let ul_pos = [-p.x + p.y for p in anchors].max()! + d_ul - 4 * step
print("!", (ur_pos - ul_pos) / 2, (ur_pos + ul_pos) / 2)

@interactor
fn judge(mut sol: Interactor):
    let n: 1..=8 = read()
    let x0: -100..=100 = read()
    let y0: -100..=100 = read()
    let anchors: [0..n](x: -100..=100, y: -100..=100) = read()
    sol.send(n)
    for p in anchors:
        sol.send(p.x, p.y)
    var cx = x0
    var cy = y0
    var queries = 0
    while true:
        let kind: Str = sol.recv()
        if kind == "!":
            let gx: Int = sol.recv()
            let gy: Int = sol.recv()
            assert gx == x0 and gy == y0
            return
        assert kind == "?"
        queries += 1
        assert queries <= 10
        let dir: Str = sol.recv()
        let k: Int = sol.recv()
        if dir == "U": cy += k
        elif dir == "D": cy -= k
        elif dir == "R": cx += k
        elif dir == "L": cx -= k
        else: assert false
        var best = 1e18
        for p in anchors:
            best min= abs(cx - p.x) + abs(cy - p.y)
        sol.send(best)
"#;

const CORPUS_CASES: &[StressCase] = &[
    StressCase {
        id: "sac22cc2p5",
        shape: "arrays + queries (@alt(program), typed)",
        good: SAC22CC2P5_GOOD,
        buggy: SAC22CC2P5_BUGGY,
        size: "6",
        good_cases: "20",
    },
    StressCase {
        id: "dmopc20c4p4",
        shape: "arrays greedy (@alt(solve) part, @gen)",
        good: DMOPC20C4P4_GOOD,
        buggy: DMOPC20C4P4_BUGGY,
        size: "6",
        good_cases: "20",
    },
    StressCase {
        id: "ccc03s4",
        shape: "strings + multitest (@alt(program), typed)",
        good: CCC03S4_GOOD,
        buggy: CCC03S4_BUGGY,
        size: "6",
        good_cases: "20",
    },
    StressCase {
        id: "btoi17p6",
        shape: "trees parent-array (@alt(program), read(gen:))",
        good: BTOI17P6_GOOD,
        buggy: BTOI17P6_BUGGY,
        size: "6",
        good_cases: "20",
    },
    StressCase {
        id: "shoi16p1",
        shape: "trees weighted (@alt(program), @gen)",
        good: SHOI16P1_GOOD,
        buggy: SHOI16P1_BUGGY,
        size: "6",
        good_cases: "20",
    },
    StressCase {
        id: "ucrpc21h",
        shape: "graphs weighted (@alt(program), @gen)",
        good: UCRPC21H_GOOD,
        buggy: UCRPC21H_BUGGY,
        size: "5",
        good_cases: "15",
    },
    StressCase {
        id: "yac7p3",
        shape: "graphs functional (@alt(program), typed)",
        good: YAC7P3_GOOD,
        buggy: YAC7P3_BUGGY,
        size: "6",
        good_cases: "20",
    },
    StressCase {
        id: "ccc04s5",
        shape: "2D grids (@alt(program), @gen)",
        good: CCC04S5_GOOD,
        buggy: CCC04S5_BUGGY,
        size: "5",
        good_cases: "20",
    },
    StressCase {
        id: "aac5p4",
        shape: "constructive (@check, typed)",
        good: AAC5P4_GOOD,
        buggy: AAC5P4_BUGGY,
        size: "12",
        good_cases: "30",
    },
    StressCase {
        id: "cf2135b",
        shape: "interactive + multitest (@interactor, typed)",
        good: CF2135B_GOOD,
        buggy: CF2135B_BUGGY,
        size: "4",
        good_cases: "10",
    },
];

/// The 10 cases are split into `SHARDS` tests by index, so each stays short.
const SHARDS: usize = 4;

fn stress_shard(shard: usize) {
    assert_eq!(CORPUS_CASES.len(), 10);
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("stress_corpus_10_{shard}"));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();

    let mut summary: Vec<(&str, &str, Duration, String)> = Vec::new();
    let mut failures: Vec<String> = Vec::new();

    for case in CORPUS_CASES.iter().skip(shard).step_by(SHARDS) {
        let dir = root.join(case.id);
        std::fs::create_dir_all(&dir).unwrap();

        // 1. Correct solution agrees on generated cases.
        let good_path = dir.join("good.cp");
        std::fs::write(&good_path, case.good).unwrap();
        let good_out = Command::new(cpl_bin())
            .args([
                "stress",
                good_path.to_str().unwrap(),
                "--seed",
                "1",
                "--cases",
                case.good_cases,
                "--max-size",
                case.size,
                "--threads",
                "2",
                "--time-limit",
                "30000",
            ])
            .output()
            .expect("cpl stress good runs");
        let good_stdout = String::from_utf8_lossy(&good_out.stdout);
        let good_stderr = String::from_utf8_lossy(&good_out.stderr);
        if !good_out.status.success() || !good_stdout.contains("cases agree") {
            failures.push(format!(
                "{} (good) failed:\n{good_stdout}\n{good_stderr}",
                case.id
            ));
            continue;
        }

        // 2. Planted bug fails with shrunk counterexample printed in input format.
        let bug_path = dir.join("buggy.cp");
        std::fs::write(&bug_path, case.buggy).unwrap();
        let start = Instant::now();
        let bug_out = Command::new(cpl_bin())
            .args([
                "stress",
                bug_path.to_str().unwrap(),
                "--seed",
                "1",
                "--cases",
                "200",
                "--max-size",
                case.size,
                "--threads",
                "2",
                "--time-limit",
                "30000",
            ])
            .output()
            .expect("cpl stress buggy runs");
        let took = start.elapsed();
        let bug_stdout = String::from_utf8_lossy(&bug_out.stdout);
        let bug_stderr = String::from_utf8_lossy(&bug_out.stderr);
        if bug_out.status.code() != Some(1)
            || !bug_stdout.contains("FAIL on case ")
            || !bug_stdout.contains("saved ")
        {
            failures.push(format!(
                "{} (buggy text) unexpected output (code {:?}):\n{bug_stdout}\n{bug_stderr}",
                case.id,
                bug_out.status.code()
            ));
            continue;
        }

        // 3. Deterministic seed check: two JSON runs with `--seed 1` produce the exact same shrunk input.
        let mut raw_inputs = Vec::new();
        for _ in 0..2 {
            let json_out = Command::new(cpl_bin())
                .args([
                    "stress",
                    bug_path.to_str().unwrap(),
                    "--seed",
                    "1",
                    "--cases",
                    "200",
                    "--max-size",
                    case.size,
                    "--threads",
                    "2",
                    "--time-limit",
                    "30000",
                    "--format",
                    "json",
                ])
                .output()
                .expect("cpl stress json runs");
            let json = String::from_utf8_lossy(&json_out.stdout).to_string();
            match json_string_field(&json, "raw_input") {
                Some(raw) if !raw.trim().is_empty() => raw_inputs.push(raw),
                _ => {
                    failures.push(format!("{} (json) missing raw_input: {json}", case.id));
                    break;
                }
            }
        }
        if raw_inputs.len() == 2 && raw_inputs[0] != raw_inputs[1] {
            failures.push(format!(
                "{} non-deterministic counterexample:\nrun1:\n{}\nrun2:\n{}",
                case.id, raw_inputs[0], raw_inputs[1]
            ));
            continue;
        }
        let compact = raw_inputs
            .first()
            .map(|s| s.trim().replace('\n', " | "))
            .unwrap_or_default();
        summary.push((case.id, case.shape, took, compact));
    }

    for (id, shape, took, counterexample) in &summary {
        println!(
            "{id:<14} {:>5.2} s  {shape:<48}  min: {counterexample}",
            took.as_secs_f64()
        );
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

#[test]
fn cpl_stress_works_end_to_end_on_10_corpus_problems_of_different_shapes_0() {
    stress_shard(0);
}

#[test]
fn cpl_stress_works_end_to_end_on_10_corpus_problems_of_different_shapes_1() {
    stress_shard(1);
}

#[test]
fn cpl_stress_works_end_to_end_on_10_corpus_problems_of_different_shapes_2() {
    stress_shard(2);
}

#[test]
fn cpl_stress_works_end_to_end_on_10_corpus_problems_of_different_shapes_3() {
    stress_shard(3);
}

/// With no C++ compiler on PATH, the file's own `@gen` fn runs in the
/// interpreter, as its solution and brute do, and its choices still shrink.
#[test]
fn cpl_stress_interprets_a_gen_fn_where_no_cpp_compiler_is() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("stress_gen_no_compiler");
    let empty = dir.join("empty");
    std::fs::create_dir_all(&empty).unwrap();
    let path = dir.join("buggy.cp");
    std::fs::write(&path, SHOI16P1_BUGGY).unwrap();
    let out = Command::new(cpl_bin())
        .args([
            "stress",
            path.to_str().unwrap(),
            "--seed",
            "1",
            "--cases",
            "200",
        ])
        .args(["--max-size", "6", "--threads", "2", "--time-limit", "30000"])
        .env("PATH", &empty)
        .output()
        .expect("cpl stress runs");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(1), "{stdout}\n{stderr}");
    assert!(stdout.contains("FAIL on case "), "{stdout}\n{stderr}");
    assert!(
        stdout.contains("to 4:\n    1 0\n    1\n    0\n"),
        "shrinks to the one-vertex tree: {stdout}"
    );
}
