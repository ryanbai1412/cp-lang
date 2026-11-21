// If s = x + y + z and t = x + y^k + z with delta = m - n > 0, then x must lie inside
// the common prefix of s and t, z inside the common suffix, and |y| must divide delta.
// Trimming the forced prefix/suffix leaves a core of length core_n where s is both a
// prefix and suffix of t, so t is delta-periodic; |y| = a works iff it is a multiple
// of the primitive period g of t's delta-block and divides delta, giving core_n - a + 1 starts.
use std.strings.period

let n: 1..=10_000_000 = read()
let m: 2..=10_000_000 = read()
let s: Str = read()
let t: Str = read()

let max_pre = (0..n).first(i => s[i] != t[i]) ?? n
let max_suf = (0..n).first(i => s[n - 1 - i] != t[m - 1 - i]) ?? n
if max_pre + max_suf <= n:
    print(0)
else:
    let min_x = n - max_suf
    let core_n = max_pre + max_suf - n
    let delta = m - n
    let block = [t[min_x + i] for i in 0..delta]
    let p = period(block)
    let g = if delta % p == 0: p else: delta
    var ans = 0
    var a = g
    while a <= core_n and a <= delta:
        if delta % a == 0: ans += core_n - a + 1
        a += g
    print(ans)
