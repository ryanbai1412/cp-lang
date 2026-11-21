use std.strings.{Hash2D}

let rows = 500
let cols = 500
domain R = 0..rows
domain C = 0..cols
var grid = [R, C] (r, c) => 0
var x = 1234567
for r in R:
    for c in C:
        x = (x * 1103515245 + 12345) & 0x7fffffff
        grid[r, c] = (x >> 16) % 26 + 97

let h = Hash2D<R, C>.new((r, c) => grid[r, c], bases: (131, 137))

var acc = 0
for q in 0..500000:
    x = (x * 1103515245 + 12345) & 0x7fffffff
    let r1 = (x >> 8) % rows
    x = (x * 1103515245 + 12345) & 0x7fffffff
    let r2 = (x >> 8) % rows
    let ra = min(r1, r2)
    let rb = max(r1, r2)
    x = (x * 1103515245 + 12345) & 0x7fffffff
    let c1 = (x >> 8) % cols
    x = (x * 1103515245 + 12345) & 0x7fffffff
    let c2 = (x >> 8) % cols
    let ca = min(c1, c2)
    let cb = max(c1, c2)
    let hv = h.rect(R(ra)..=R(rb), C(ca)..=C(cb))
    acc ^= hv.h1 ^ (hv.h2 << 1)
print(acc)
