// Codeforces 2222 D: sort prefix sums of A[0..N-1] and assign descending ranks
// 1..N to their indices.
@!multitest
let n: Int = read()
let a: [0..n]Int = read()
let pref = prefix(a)
let ord = (0..n).sorted(by: i => (pref.prefix(i), i))
var p: [0..n]Int = 0
for (rank, idx) in ord.items: p[idx] = n - rank
print(p)
