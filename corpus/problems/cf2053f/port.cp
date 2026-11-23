// Source: Codeforces 2053F (Earnest Matrix Complement),
//   https://codeforces.com/contest/2053/problem/F
// Problem: an n x m matrix (n, m >= 2, sum of n m <= 6e5; multitest) of values in 1..=k with
//   blanks -1. Fill the blanks to maximise sum over u and i of c[u, i] c[u, i + 1], where
//   c[u, i] counts u in row i; print the maximum.
// Solution: some optimum gives all blanks of a row one value. Credit every adjacent row pair up
//   front with b_i b_{i+1} (blanks agreeing) plus its fixed-value matches; a DP over rows then
//   picks each row's value v, gaining b_i times the count of v in rows i - 1 and i + 1, and pays
//   b_i b_{i+1} back when the value changes. stay[v] is the best ending on v, fresh the best after
//   a paid change; only values present in a neighbour row are touched, and `best` (max stay)
//   only rises. O(n m) hash operations per test.
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
    for j in [i.ord - 1, i.ord + 1]:
        if j in R:
            for (v, c) in cnt[R(j)]:
                stay[v] = max(stay[v], fresh) + c * blank[i]
                best max= stay[v]
    if let next = i.next:
        credit += blank[i] * blank[next] + sum(c * (cnt[next].get(v) ?? 0) for (v, c) in cnt[i])
        fresh max= best - blank[i] * blank[next]
print(credit + max(fresh, best))
