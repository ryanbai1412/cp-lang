// Batch #15: a list for `[1..=3]T`, `last(k)` on arrays and `Deque`, part of a row or column.
use std.deque
let c: [1..=3]Int = [7, 8, 9]
print(c[1], c[3])
let n: Int = read()
var e: [0..n]Int = [1, 2, 3]
let d: [2..n + 1]Int = [5, 6]
print(d[2], d[3])
e[0] += 10
print(e)
let f: [-1..=1]Str = ["a", "b", "c"]
print(f[-1], f[1])
domain I = 0..3
fn fmt_opt(x: ?Int) -> Str = if let v = x: "{v}" else: "none"
let a: [I]Int = [4, 5, 6]
print(fmt_opt(a.last(0)), fmt_opt(a.last(1)), fmt_opt(a.last(5)))
print(fmt_opt(c.last(1)), fmt_opt(c.last(2)), fmt_opt(c.last(3)))
var dq = Deque<Int>()
dq.push(1)
dq.push(2)
print(fmt_opt(dq.last(0)), fmt_opt(dq.last(1)), fmt_opt(dq.last(2)), fmt_opt(dq.last))
var g: [0..3, 0..4]Int = 0
for r in 0..3:
    for c in 0..4: g[r, c] = r * 4 + c + 1
print(g[1, 1..3])
print(g[0..2, 3])
print(g[1, ..])
print(g[2, 2..])
print(g[..=1, 0])
domain R = 0..3
domain C = 0..4
var h: [R, C]Int = 0
for r in R:
    for c in C: h[r, c] = r.idx * 10 + c.idx
let mid: C = C.first!.next!
print(h[R.first!, mid..], h[R.first!.., C.last!], h[R.last!, ..=mid])
var k: [1..=2, 1..=3]Int = 0
k[2, 3] = 7
print(k[2, 2..=3], k[1..=2, 3])
print(g[0, n..n], g[0, 1..n])
for (i, x) in a.items: print(a[i] + x, end: " ")
print()
