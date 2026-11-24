// fixed: k
use std.seq.{iterate, product, span, kway_merge, compress}
let n: Int = read()
let xs = [i * 7919 % n for i in 0..n]
let ys = [i for i in 0..n]
domain K = 0..256
let twos = [(i, i * 2) for i in 0..n]
let rs = [i..i + 2 for i in 0..n]
let half = (x: Int) => x >= n / 2 // within: seq.bsearch
let big = (t: Int) => t >= n / 2 // within: seq.pop_while
let digit = (x: Int) => K.nth(x % 256) // within: seq.counting_sort
let next = (x: Int) => if x + 1 < n: Some(x + 1) else: None // within: seq.iterate
let hi = ys.bsearch(half)
var s = 0
for r in 0..262144:
    let t = r % n
    s += ys.take(5).len // timed: seq.take
    s += ys.drop(5).len // timed: seq.drop
    s += ys.drop_last(5).len // timed: seq.drop_last
    s += ys.take_last(5).len // timed: seq.take_last
    s += ys.nth(t) // timed: seq.nth
    s += ys.first_gt(t) ?? 0 // timed: seq.first_gt
    s += ys.last_lt(t) ?? 0 // timed: seq.last_lt
    s += ys.equal_range(t).len // timed: seq.equal_range
    s += ys.count_in(t..n) // timed: seq.count_in
    s += span(ys.bsearch(half), hi).len // timed: seq.bsearch
for r in 0..256:
    s += Int(xs.majority().is_some_and(m => m.count > 0)) // timed: seq.majority
for r in 0..64:
    var a = xs.clone()
    let (l, m) = twos.unzip() // timed: seq.unzip
    s += l.len + m.len
    s += rs.outermost().len // timed: seq.outermost
    s += kway_merge([ys, ys]).len // timed: seq.kway_merge
    a.counting_sort(by: digit) // timed: seq.counting_sort
    a.select(n / 2) // timed: seq.select
    var b = ys.clone()
    s += b.pop_while(big).len // timed: seq.pop_while
    for v in iterate(from: Some(0), step: next): // timed: seq.iterate
        s += v // timed: seq.iterate
    for c in ys.chunks_padded(8, pad: 0): // timed: seq.chunks_padded
        s += c.len // timed: seq.chunks_padded
    for (u, v) in product(ys.take(128), ys.take(n / 128)): // timed: seq.product
        s += u ^ v // timed: seq.product
domain W = 0..n
for r in 0..262144:
    s += span(W.cuts.first!, W.cuts.last!).len // timed: seq.span
print(s)
