// Codeforces 2053 F: fill the blanks of a matrix with 1..=k, maximizing Σ_v Σ_i c[v, i]·c[v, i + 1].
// Some optimum gives all blanks of a row one value. Credit every pair of adjacent blank rows up
// front; a DP over the value of the current row's blanks pays b_i·b_{i + 1} back only when the
// value changes, and only values present in a neighbouring row ever rise, so `best` is a running max.
@!multitest
let (n, m, k) = read<(2..=2e5, 2..=2e5, Int)>()
domain R = 0..n
domain V = 1..=k
let a: [R][m]?V = read(none: -1)
let blank = [R] i => a[i].count(none)
let cnt = [R] i => a[i].present.tally()

var credit = sum(blank[i] * blank[j] + cnt[i].dot(cnt[j]) for (i, j) in R.range.pairs())
var stay: [V]Int = 0
var fresh = 0
var best = 0
for i in R:
    for j in [i.prev, i.next].flatten(), (v, c) in cnt[j]:
        stay[v] = max(stay[v], fresh) + c * blank[i]
        best max= stay[v]
    if let j = R.next(i): fresh max= best - blank[i] * blank[j]
print(credit + max(fresh, best))
