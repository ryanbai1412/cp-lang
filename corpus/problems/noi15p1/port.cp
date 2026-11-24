// Source: NOI 2015 P1 (Automated Program Analyzer), https://dmoj.ca/problem/noi15p1
// limits: 2000 ms, 512 MB
// Problem: t independent instances, each n <= 1e5 constraints x_i = x_j or x_i != x_j with
//   indices up to 1e9. For each, print YES if all its constraints can hold at once, else NO.
// Solution: compress the indices, unite the two sides of every equality in a Dsu, then check
//   that no inequality has both sides in one class. O(n log n) per instance.
// original: corpus/submissions/3775616.cpp

@multitest
let q: Int = read()
var rules: List<(a: Int, b: Int, same: Bool)> = []
var raw: List<Int> = []
for _ in 0..q:
    let (i, j, e): (Int, Int, Int) = read()
    raw.push(i)
    raw.push(j)
    rules.push((a: i, b: j, same: e == 1))

let (domain Var, vars) = compress(raw)
var cls = Dsu<Var>()
for r in rules:
    if r.same: cls.unite(vars.index_of(r.a), vars.index_of(r.b))
let ok = rules.filter(r => not r.same).all(r => not cls.same(vars.index_of(r.a), vars.index_of(r.b)))
print(if ok: "YES" else: "NO")
