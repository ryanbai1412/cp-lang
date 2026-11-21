// Codeforces 2187 A: sort (value, index) pairs and minimize max distance to the
// global min/max over all elements not already in sorted position.
@!multitest
let n: Int = read()
let a: [0..n]Int = read()
let s = sorted([(a[i], i) for i in 0..n])
let (lo, hi) = (s[0].0, s.last!.0)
print(min(max(x - lo, hi - x) for (i, (x, j)) in s.items if j != i) ?? -1)
