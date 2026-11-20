// SHOI 2016 P4, Black-box company: n cities and n - 1 companies, each
// able to build some roads. Count the spanning trees in which every
// company builds exactly one road, modulo 1e9 + 7.
//
// Inclusion-exclusion over the companies left out: a tree of n - 1 roads
// that avoids a set S of companies is counted with sign (-1)^|S|, and
// what survives is the trees using every company. Trees of a multigraph
// are a determinant (matrix-tree theorem).
// input: n, then per company its road count m and m roads `u v`
// original: corpus/submissions/5635192.cpp
use std.mod.{Mod}
type Z = Mod<1_000_000_007>

let n: Int = read()
domain City = 1..=n
domain Company = 1..n
domain Rest = 1..n          // every city but the last
var roads: [Company]List<(u: City, v: City)> = [Company] c => []
for c in Company:
    let m: Int = read()
    for _ in 0..m:
        let e: (u: City, v: City) = read()
        roads[c].push(e)

/// Determinant of `a`, by elimination.
fn det(mut a: [Rest, Rest]Z) -> Z:
    var ans = Z(1)
    for i in Rest:
        var pivot: ?Rest = none
        for r in i..:
            if a[r, i] != Z(0):
                pivot = r
                break
        if pivot == none: return Z(0)
        let p = pivot!
        if p != i:
            for c in Rest: (a[i, c], a[p, c]) = (a[p, c], a[i, c])
            ans = -ans
        ans = ans * a[i, i]
        let inv = a[i, i].inv
        for r in i.r..:
            let f = a[r, i] * inv
            for c in i..: a[r, c] -= f * a[i, c]
    ans

var total = Z(0)
for skipped in 0..(1 << Company.K):
    // Laplacian of the roads of the companies not skipped, without the
    // row and column of the last city.
    var lap: [Rest, Rest]Z = [Rest, Rest] (i, j) => Z(0)
    for c in Company:
        if ((skipped >> c.ord) & 1) == 1: continue
        for e in roads[c]:
            if e.u == e.v: continue
            if let u = Rest.try(e.u.label): lap[u, u] += Z(1)
            if let v = Rest.try(e.v.label): lap[v, v] += Z(1)
            if let u = Rest.try(e.u.label):
                if let v = Rest.try(e.v.label):
                    lap[u, v] -= Z(1)
                    lap[v, u] -= Z(1)
    let trees = det(mut lap)
    if popcount(skipped) % 2 == 0: total += trees else: total -= trees
print(total)
