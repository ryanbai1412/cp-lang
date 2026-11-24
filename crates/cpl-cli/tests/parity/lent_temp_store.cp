// A view stored into a field, a list or a reassigned variable owns its temporaries.
use std.seq.{zip}
struct H: xs: List<Int>
struct Box: var h: H
fn wrap(xs: List<Int>) -> lends H = H(xs)
fn fill(n: Int) -> Int:
    var b = Box(wrap([1]))
    var hs: List<H> = []
    for i in 0..n:
        b.h = wrap([i for _ in 0..i + 1])
        hs.push(wrap([j for j in 0..i + 1]))
    var t = 0
    for h in hs: t += h.xs.len
    b.h.xs.len * 1000 + t
let n: Int = read()
var w = zip([0], [0])
for i in 1..n + 1: w = zip([j for j in 0..i], [j * 2 for j in 0..i + 1])
print(fill(n), w.len, w.nth(n - 1).1)
