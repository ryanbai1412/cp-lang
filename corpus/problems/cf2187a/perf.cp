// Tuned: each (value, index) pair packed into one Int key, value << 18 | index (n <= 2e5 <
// 2^18), so the sort is one integer sort of 8-byte keys like orig.cpp's pair<int, int>, not
// a merge sort of tuples; the answer is folded straight off the sorted keys.
@!multitest
let n: 1..=200_000 = read()
let a: [0..n]1..=10 ** 9 = read()
let s = sorted([a[i] << 18 | i for i in 0..n])
let (lo, hi) = (s[0] >> 18, s.last! >> 18)
print(min(max((k >> 18) - lo, hi - (k >> 18)) for (i, k) in s.items if k % (1 << 18) != i) ?? -1)
