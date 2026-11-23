// Codeforces 2085D Serval and Kaitenzushi Buffet: https://codeforces.com/problemset/problem/2085/D
// A set of take-minutes is feasible iff every suffix j..n holds at most (n - j + 1) / (k + 1) of them.
// Those sets form a matroid, so take plates by deliciousness, best first, while the suffix slack allows:
// slack[j] = quota(j) - taken in j..n; taking minute i spends one slack on every j <= i.
// Revision: also print the minutes he takes plates at, in increasing order.
@!multitest
let (n, k): (2..=2e5, 1..=n - 1) = read()
domain T = 1..=n
let d = read<[T]1..=1e9>()
var slack = SegTree(min, [T] j => (n - Int(j) + 1) / (k + 1), act: add)
var total = 0
var taken = List<T>()
for i in T.sorted(by: i => desc(d[i])):
    if slack.fold(..=i) >= 1:
        slack.apply(..=i, -1)
        total += d[i]
        taken.push(i)
print(total)
print(taken.sorted())
