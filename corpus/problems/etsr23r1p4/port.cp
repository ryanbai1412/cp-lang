// Source: ETSR '23 R1 P4 (LOJ 6029, Market), https://dmoj.ca/problem/etsr23r1p4
// Problem: 1 <= n, q <= 2e5 and an array a_1..a_n of 32-bit ints. Queries "1 l r c" add
//   -1e4 <= c <= 1e4 to a_l..=a_r, "2 l r d" replaces each by floor(a_i / d) for 1 <= d <= 1e9,
//   "3 l r" prints the minimum and "4 l r" the sum of a_l..=a_r.
// Solution: segment tree beats. Floor division is an add on a node once its min and max move by
//   the same amount, which happens after O(log) divisions of a node's spread, so BeatsSegTree's
//   chdiv is amortised O((n + q) log n log V) with add, min and sum as usual.
// original: corpus/submissions/5495163.cpp

let (n, q): (1..=2e5, 1..=2e5) = read()
domain I = 1..=n
let a: [I]-2147483648..=2147483647 = read()
var t = BeatsSegTree([I] i => Int(a[i]))
for _ in 0..q:
    let (op, l, r): (1..=4, I, I) = read()
    if op == 1: t.add(l..=r, read<-1e4..=1e4>())
    elif op == 2: t.chdiv(l..=r, read<1..=1e9>())
    elif op == 3: print(t.min_val(l..=r)!)
    else: print(t.sum(l..=r))
