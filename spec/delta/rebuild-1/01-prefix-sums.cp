// cpl 0.3.0
// Problem: static range sums, multitest.
// Input: t; per test: n q, then a_1..a_n (|a_i| <= 1e9), then q lines
// "l r" with 1 <= l <= r <= n. Output: a_l + ... + a_r per query.
// Guesses: H-NEG-RANGE-TYPE (`[V]-1e9..=1e9` parses), H-PREFIX-INDEX.

@!multitest
let n = read<1..=2e5>()
let q = read<1..=2e5>()
domain V = 1..=n
let a = read<[V]-1e9..=1e9>()
let pre = a.scan(+)
for _ in 0..q:
    let (l, r) = read<(V, V)>()
    print(pre[r] - pre[l.l])
