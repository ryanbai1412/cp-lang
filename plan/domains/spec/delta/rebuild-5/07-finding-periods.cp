// CSES 1733 Finding Periods: every length p such that s is a prefix of (s[..p]) repeated.
// Input: one string s. Output: all periods in increasing order.
let s = read<Str(len: 1..=1e6)>()
let n = s.len
let z = z_function(s)
print([p for p in 1..=n if p == n or z[p] == n - p])
