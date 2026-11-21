use std.strings.{SuffixArray}

let n = 200_000
var s = List.filled(n, 0)
var x = 135792468
for i in 0..n:
    x = (x * 1103515245 + 12345) & 0x7fffffff
    s[i] = (x >> 16) % 4
let sa = SuffixArray.new(s)
var ans = sa.distinct
for q in 0..200_000:
    x = (x * 1103515245 + 12345) & 0x7fffffff
    let a = (x >> 8) % n
    x = (x * 1103515245 + 12345) & 0x7fffffff
    let b = (x >> 8) % n
    ans += sa.lcp_between(a, b)
print(ans)
