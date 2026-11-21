use std.strings.{RollingHash}

let n = 500_000
var s = List.filled(n, 0)
var x = 987654321
for i in 0..n:
    x = (x * 1103515245 + 12345) & 0x7fffffff
    s[i] = (x >> 16) % 4
let rh = RollingHash.new(s, bases: (911382323, 972663749))
var ans = 0
for q in 0..200_000:
    x = (x * 1103515245 + 12345) & 0x7fffffff
    let a = (x >> 8) % n
    x = (x * 1103515245 + 12345) & 0x7fffffff
    let b = (x >> 8) % n
    let l = min(a, b)
    let r = max(a, b)
    let h = rh.slice(l, r)
    ans ^= h.h1 + h.h2 + rh.lcp(a, b)
print(ans)
