// If s = x + y + z and t = x + y^k + z with delta = m - n > 0, then x must lie inside
// the common prefix of s and t, z inside the common suffix, and |y| must divide delta.
// Trimming the forced prefix/suffix leaves a core of length core_n where s is both a
// prefix and suffix of t, so t is delta-periodic; |y| = a works iff it is a multiple
// of the primitive period g of t's delta-block and divides delta, giving core_n - a + 1 starts.
use std.strings.period

let (n, m): (1..=10_000_000, 2..=10_000_000) = read()
let (s, t): (Str, Str) = read()

let max_pre = s.common_prefix_len(t)
let max_suf = s.common_suffix_len(t)
if max_pre + max_suf <= n:
    print(0)
    return

let min_x = n - max_suf
let core_n = max_pre + max_suf - n
let delta = m - n
let p = period(t[min_x..min_x + delta])
let g = if delta % p == 0: p else: delta

print([core_n - a + 1 for a in g..=min(core_n, delta) by g if delta % a == 0].sum())
