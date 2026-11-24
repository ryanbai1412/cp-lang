// Source: Codeforces 2180E (No Effect XOR), https://codeforces.com/contest/2180/problem/E
// limits: 2000 ms, 256 MB
// Problem: count the positive x such that i XOR x lies in [l, r] for every i in [l, r]
//   (1 <= l <= r <= 1e15). Multitest, t <= 1e5.
// Solution: use the half-open [l, r + 1). Any x below 2^b, b = min(ctz(l), ctz(r + 1)),
//   permutes the aligned 2^b blocks the range is made of; other low bits push a boundary
//   block out. The top bit where l and r + 1 differ splits the range at an aligned midpoint,
//   and flipping it works iff both halves are equally long. So the answer is
//   (1 or 2) * 2^b - 1, excluding x = 0. O(1).

@!multitest
let (l, r0): (1..=2 ** 60, 1..=2 ** 60) = read()
let r = r0 + 1
let half = 1 << (bit_width(l ^ r) - 1)
let sym = (l & (half - 1)) + (r & (half - 1)) == half
print(((1 + Int(sym)) << min(ctz(l), ctz(r))) - 1)
