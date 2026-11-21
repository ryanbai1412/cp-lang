use std.seq.{scan, suffix_scan}

let n = 10_000
domain Pos = 0..n
var seed = 123456789
var a: [Pos]Int = 0
for i in Pos:
    seed = (seed * 1103515245 + 12345) % 2147483647
    a[i] = (seed % 2001) - 1000

var acc = 0
for r in 0..2000:
    let ps = a.scan(+)
    let pm = a.scan(min)
    let sm = a.suffix_scan(max)
    acc += ps[Pos(n - 1 - (r % n))] + pm[Pos(n - 1 - (r % n))] + sm[Pos(r % n)]

print(acc)
