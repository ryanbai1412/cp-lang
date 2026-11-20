// NOI 2015 Automated Program Analyzer: each test is a list of constraints
// `x[i] == x[j]` or `x[i] != x[j]` over variables numbered up to 1e9.
// Print YES if all of them can hold at once.
//
// Unite the equalities, then no inequality may sit inside one class. The
// variable numbers are sparse, so they are compressed to a domain first.
// original: corpus/submissions/3775616.cpp
use std.dsu.{Dsu}

fn consistent() -> Bool:
    let q: Int = read()
    var ids = Map<Int, Int>()
    var rules: List<(a: Int, b: Int, same: Bool)> = []
    for _ in 0..q:
        let i: Int = read()
        let j: Int = read()
        let e: Int = read()
        if i not in ids: ids[i] = ids.len
        if j not in ids: ids[j] = ids.len
        rules.push((a: ids[i], b: ids[j], same: e == 1))
    domain Var = 0..ids.len
    var cls = Dsu<Var>.new()
    for r in rules:
        if r.same: cls.unite(Var(r.a), Var(r.b))
    for r in rules:
        if not r.same and cls.same(Var(r.a), Var(r.b)): return false
    true

let t: Int = read()
for _ in 0..t:
    print(if consistent(): "YES" else: "NO")
