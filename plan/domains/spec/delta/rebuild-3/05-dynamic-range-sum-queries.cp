// CSES 1648 Dynamic Range Sum Queries: point assignment and range sum.
// In: n q, x1..xn, then q queries "1 k u" (x_k = u) or "2 a b" (sum of a..=b). Out: each sum.
domain I = 1..=read<1..=2e5>()
let q = read<1..=2e5>()
var sums = Fenwick(read<[I]1..=1e9>())

enum Query:
    @tag(1) Set(k: I, u: 1..=1e9)
    @tag(2) Sum(r: Range<I>)

for _ in 0..q:
    match read<Query>():
        Set(k, u) => sums.set(k, u)
        Sum(r) => print(sums.fold(r))
