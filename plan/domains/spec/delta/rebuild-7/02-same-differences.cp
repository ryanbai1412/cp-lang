// Codeforces 1520D Same Differences
// in: t; per test n, then a_1..a_n (1 <= a_i <= n). out: per test the number of pairs i < j with a_j - a_i = j - i.
@!multitest
domain I = 1..=read<1..=2e5>()
let a = read<[I]1..=2e5>()
let same = I.tally(by: i => a[i] - Int(i))
print(sum(choose2(c) for c in same.values))
