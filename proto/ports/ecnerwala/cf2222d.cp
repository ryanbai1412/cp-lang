// Codeforces 2222 D: sort prefix sums of A[0..N-1] and assign descending ranks
// 1..N to their indices.
@!multitest
let n: Int = read()
let a: [0..n]Int = read()
var pref: List<(Int, Int)> = [(0, 0)]
var run = 0
for i in 0..n - 1:
    run += a[i]
    pref.push((run, i + 1))
let ord = sorted(pref)
var p: [0..n]Int = 0
for i in 0..n: p[ord[i].1] = n - i
print(p)
