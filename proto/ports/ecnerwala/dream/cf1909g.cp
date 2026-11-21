// If s = x + y + z and t = x + y^k + z with delta = m - n > 0, then x must lie inside
// the common prefix of s and t, z inside the common suffix, and |y| must divide delta.
// Trimming the forced prefix/suffix leaves a core of length core_n where s is both a
// prefix and suffix of t, so t is delta-periodic; |y| = a works iff it is a multiple
// of the primitive period g of t's delta-block and divides delta, giving core_n - a + 1 starts.
use std.strings.period

let (n, m): (1..=10_000_000, 2..=10_000_000) = read()
let (s, t): (Str, Str) = read()

let max_pre = first(i for i in 0..n if s[i] != t[i]) ?? n
let max_suf = first(i for i in 0..n if s[n - 1 - i] != t[m - 1 - i]) ?? n
if max_pre + max_suf <= n:
    print(0)
    return

let min_x = n - max_suf
let core_n = max_pre + max_suf - n
let delta = m - n
let block = t[min_x..min_x + delta]
let p = period(block)
let g = if delta % p == 0: p else: delta

print(sum(core_n - a + 1 for a in g..=min(core_n, delta) by g if delta % a == 0))
