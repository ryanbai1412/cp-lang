use std.strings.{PalindromicTree}

domain Alpha = 0..4
let n = 1_000_000
var s = List.filled(n, Alpha(0))
var x = 135792468
for i in 0..n:
    x = (x * 1103515245 + 12345) & 0x7fffffff
    s[i] = Alpha((x >> 16) & 3)
let pt = PalindromicTree<Alpha>.of(s)
let occ = pt.occurrences()
var ans = pt.distinct
for i in 2..occ.len:
    ans += occ[i] * pt.pal_len(i)
print(ans)
