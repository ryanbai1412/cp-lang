use std.fenwick.{RangeFenwick}

let n = 200_000
domain I = 0..n
var rf = RangeFenwick<I>()
var s = 0
var x = 1234567
for _ in 0..200_000:
    x = (x * 1103515245 + 12345) & 2147483647
    let l = x % n
    x = (x * 1103515245 + 12345) & 2147483647
    let r = x % n
    let (a, b) = (min(l, r), max(l, r))
    rf.add(I(a)..=I(b), (x % 17) - 8)
    s += rf.sum(I(a)..=I(b))
print(s)
