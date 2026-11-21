// SHOI 2016 P4, Black-box Company (CP Lang 0.2.2): inclusion-exclusion over
// `Subset<Company>` with Kirchoff matrix-tree determinant in `Mod<1_000_000_007>`.
// original: corpus/submissions/5635192.cpp
use std.mod.{Mod}

type Z = Mod<1_000_000_007>

let n: Int = read()
domain City = 1..=n
domain Company = 1..n
domain Rest = 1..n
var roads: [Company]List<(u: City, v: City)> = []
for c in Company:
    let m: Int = read()
    for _ in 0..m:
        let e: (u: City, v: City) = read()
        roads[c].push(e)

fn det(mut a: [Rest, Rest]Z) -> Z:
    var ans = Z.one
    for i in Rest:
        var pivot: ?Rest = none
        for r in i..:
            if a[r, i] != Z.zero:
                pivot = r
                break
        let p = pivot ?? return Z.zero
        if p != i:
            for c in Rest: (a[i, c], a[p, c]) = (a[p, c], a[i, c])
            ans = -ans
        ans *= a[i, i]
        let inv = a[i, i].inv
        for r in i.r..:
            let f = a[r, i] * inv
            for c in i..: a[r, c] -= f * a[i, c]
    ans

var total = Z.zero
for skipped in Subset<Company>:
    var lap: [Rest, Rest]Z = 0
    for c in Company:
        if c in skipped: continue
        for e in roads[c]:
            if e.u == e.v: continue
            if let u = Rest.from_idx(e.u.label): lap[u, u] += Z.one
            if let v = Rest.from_idx(e.v.label): lap[v, v] += Z.one
            if let u = Rest.from_idx(e.u.label):
                if let v = Rest.from_idx(e.v.label):
                    lap[u, v] -= Z.one
                    lap[v, u] -= Z.one
    let trees = det(mut lap)
    if popcount(skipped.idx) % 2 == 0: total += trees else: total -= trees
print(total)
