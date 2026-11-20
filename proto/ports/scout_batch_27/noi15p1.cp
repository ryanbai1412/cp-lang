// NOI 2015 Automated Program Analyzer (CP Lang 0.2.2): check satisfiability of
// equality and disequality constraints over sparse variable IDs via `compress` + `Dsu`.
// original: corpus/submissions/3775616.cpp
use std.dsu.{Dsu}
use std.seq.{compress}

@multitest
let q: Int = read()
var rules: List<(a: Int, b: Int, same: Bool)> = []
var raw: List<Int> = []
for _ in 0..q:
    let (i, j, e): (Int, Int, Int) = read()
    raw.push(i)
    raw.push(j)
    rules.push((a: i, b: j, same: e == 1))

let vars = compress(raw)
type Var = vars.D
var cls = Dsu<Var>()
for r in rules:
    if r.same: cls.unite(vars.at(r.a), vars.at(r.b))
let ok = all(not cls.same(vars.at(r.a), vars.at(r.b)) for r in rules if not r.same)
print(if ok: "YES" else: "NO")
