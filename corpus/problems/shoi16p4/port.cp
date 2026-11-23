// Source: SHOI 2016 P4 (Darkness Before Fantasyland), https://dmoj.ca/problem/shoi16p4
// Problem: n <= 17 cities and n - 1 companies, each able to build its own list of roads.
//   Count, mod 1e9+7, the spanning trees whose n - 1 edges come from distinct companies (each
//   company builds exactly one), as sets of (road, company) choices.
// Solution: inclusion-exclusion over the set of companies left out: (-1)^|skipped| times the
//   spanning trees of the remaining companies' roads, by the matrix-tree theorem (Laplacian
//   minus city n, Gaussian determinant in Mod). O(2^(n-1) n^3).
// original: corpus/submissions/5635192.cpp
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
        var pivot: ?Rest = None
        for r in Rest if r >= i:
            if a[r, i] != Z.zero:
                pivot = r
                break
        let p = pivot ?? return Z.zero
        if p != i:
            for c in Rest: (a[i, c], a[p, c]) = (a[p, c], a[i, c])
            ans = -ans
        ans *= a[i, i]
        let inv = a[i, i].inv()
        for r in Rest if r > i:
            let f = a[r, i] * inv
            for c in Rest if c >= i: a[r, c] -= f * a[i, c]
    ans

var total = Z.zero
for skipped in Subset<Company>:
    var lap: [Rest, Rest]Z = 0
    for c in Company:
        if c in skipped: continue
        for e in roads[c]:
            if e.u == e.v: continue
            let (u, v) = (Rest.at(e.u.label), Rest.at(e.v.label))
            if let u = u: lap[u, u] += 1
            if let v = v: lap[v, v] += 1
            if let u = u, let v = v:
                lap[u, v] -= 1
                lap[v, u] -= 1
    let trees = det(mut lap)
    if skipped.len % 2 == 0: total += trees else: total -= trees
print(total)
