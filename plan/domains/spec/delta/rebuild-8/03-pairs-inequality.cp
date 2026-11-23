// 1703F Yet Another Problem About Pairs Satisfying an Inequality
// in: t; n; a_1..a_n. out: per test, the number of pairs i < j with a_i < i < a_j < j.
@!multitest
let n = read<2..=2e5>()
domain V = 1..=n
let a = read<[V]0..=1e9>()
let good = [V] i => a[i] < Int(i)
let good_before = good.map(b => if b: 1 else: 0).scan(+)
print(sum(good_before[V.cuts(max(a[j] - 1, 0))] for j in V if good[j]))
