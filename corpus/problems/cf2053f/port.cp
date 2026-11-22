// Codeforces 2053 F: fill the blanks (-1) of a matrix with 1..=k, maximizing Σ_v Σ_i c[v, i]·c[v, i + 1].
// Some optimum gives all blanks of a row one value. Credit every pair of adjacent blank rows up
// front; a DP over the value of the current row's blanks pays b_i·b_{i + 1} back only when the
// value changes, and only values present in a neighbouring row ever rise, so `best` is a running max.
@!multitest
let (n, m, k): (2..=200_000, 2..=200_000, Int) = read()
domain R = 0..n
domain V = 1..=k
let a: [R][0..m]Int = read()

let blank = [R] i => a[i].count(x => x == -1)
var cnt: [R]HashMap<V, Int> = []
for i in R:
    for x in a[i]:
        if x != -1: cnt[i][V(x)] = (cnt[i].get(V(x)) ?? 0) + 1

var credit = 0
var stay: [V]Int = 0
var (fresh, best) = (0, 0)
for i in R:
    for j in [i.to_idx - 1, i.to_idx + 1]:
        if j in R:
            for (v, c) in cnt[R(j)]:
                stay[v] = max(stay[v], fresh) + c * blank[i]
                best max= stay[v]
    if i.to_idx + 1 in R:
        let next = R(i.to_idx + 1)
        credit += blank[i] * blank[next] + sum(c * (cnt[next].get(v) ?? 0) for (v, c) in cnt[i])
        fresh max= best - blank[i] * blank[next]
print(credit + max(fresh, best))
