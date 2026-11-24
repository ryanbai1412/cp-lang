// Source: SHOI 2016 P4 (Darkness Before Fantasyland), https://dmoj.ca/problem/shoi16p4
// limits: 2000 ms, 250 MB
// Problem: 2 <= n <= 17 cities and n - 1 companies, each able to build its own list of roads
//   (no loops, no repeats).
//   Count, mod 1e9+7, the spanning trees whose n - 1 edges come from distinct companies (each
//   company builds exactly one), as sets of (road, company) choices.
// Solution: inclusion-exclusion over the set of companies left out: (-1)^|skipped| times the
//   spanning trees of the remaining companies' roads, by the matrix-tree theorem (Laplacian
//   minus city n, std Mat.det in Mod). O(2^(n-1) n^3).
// original: corpus/submissions/5635192.cpp
// Bounds: n: 2..=17; roads.u, roads.v: 1..=n
type Z = Mod<1_000_000_007>

let n: 2..=17 = read()
domain City = 1..=n
domain Company = 1..n
domain Rest = 1..n  // every city but n: the Laplacian minor
let roads: [Company]List<(u: City, v: City)> = read()

var total = Z(0)
for skipped in Subset<Company>:
    var lap: [Rest, Rest]Z = 0
    for c in Company:
        if c in skipped: continue
        for (u, v) in roads[c]:
            let (ru, rv) = (Rest.at(u.label), Rest.at(v.label))
            if let x = ru: lap[x, x] += 1
            if let y = rv: lap[y, y] += 1
            if let x = ru, let y = rv: (lap[x, y], lap[y, x]) = (lap[x, y] - 1, lap[y, x] - 1)
    let trees = Mat.new(lap).det()
    total += if skipped.len % 2 == 0: trees else: -trees
print(total)
