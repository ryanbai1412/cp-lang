use std.seq.{scan_in_place, suffix_scan_in_place}

let n = 200_000
var v: List<Int> = [0 for _ in 0..n]
var seed = 123456789
for i in 0..n:
    seed = (seed * 1103515245 + 12345) % 2147483647
    v[i] = (seed % 2001) - 1000

var acc = 0
for r in 0..400:
    v.scan_in_place(min)
    acc += v[n - 1 - r]
    v.suffix_scan_in_place(max)
    acc += v[r]
    v[r] = r - 1000

print(acc)
