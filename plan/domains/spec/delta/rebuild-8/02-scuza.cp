// 1742E Scuza
// in: t; n q; a_1..a_n (step heights); k_1..k_q (leg lengths). out: per test, q reachable heights on one line.
@!multitest
let (n, q) = read<(1..=2e5, 1..=2e5)>()
domain Step = 0..n
let a = read<[Step]1..=1e9>()
let tallest = a.scan(max)
let height = a.scan(+)
let legs = read<[q]0..=1e9>()
print([height[a.keys.bsearch(i => tallest[i] > k)] for k in legs])
