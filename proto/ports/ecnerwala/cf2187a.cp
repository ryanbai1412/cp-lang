// Codeforces 2187 A: sort (value, index) pairs and minimize max distance to the
// global min/max over all elements not already in sorted position.
@!multitest
let n: Int = read()
let a: [0..n]Int = read()
let s = sorted([(a[i], i) for i in 0..n])
let lo = s[0].0
let hi = s.last!.0

var ans: ?Int = none
for i in 0..n:
    if s[i].1 != i:
        let d = max(s[i].0 - lo, hi - s[i].0)
        ans = if let cur = ans: min(cur, d) else: d
print(ans ?? -1)
