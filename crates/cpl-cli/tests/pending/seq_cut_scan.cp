// `scan` and `suffix_scan` work directly on cut-indexed arrays `[V.cuts]T`.
use std.seq.suffix_scan

domain V = 0..4
type Cut = V.cuts

let raw = [5, 2, 8, 1, 6]
let a: [Cut]Int = [Cut] c => raw[c.ord]
let p: [V.cuts]Int = a.scan(min)
let s: [Cut]Int = a.suffix_scan(max)

print(p)
print(s)
print(p[V(2).l], p[V(2).r], s[V(0).l], s[V(3).r])
