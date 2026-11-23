// Codeforces 1324D Pair of Topics
// in: n, then a_1..a_n, then b_1..b_n (one test). out: the number of pairs i < j with a_i + a_j > b_i + b_j.
domain I = 0..read<2..=2e5>()
let a = read<[I]1..=1e9>()
let b = read<[I]1..=1e9>()
let c = I.map(i => a[i] - b[i]).sorted()
print(sum(c.len - (i + 1..c.len).bsearch(j => c[j] > -c[i]) for i in c.keys))
