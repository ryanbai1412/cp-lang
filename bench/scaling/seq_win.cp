use std.hashmap.{Counter}
use std.window.{track}
use std.seq.{first, last, filter}
let n: Int = read()
let xs = [i * 7919 % n for i in 0..n]
let ys = [i for i in 0..n]
let zs = [i % 3 for i in 0..n]
let ws = [i % 5 for i in 0..n]
domain K = 0..64
let (domain I, kv) = [K.nth(i % 64) for i in 0..n].with_domain()
let cs = [i * 16 for i in 1..n / 16]
let (domain P, piece) = (0..n).pieces(cuts: cs)
let pos = (x: Int) => x >= 0
let fr = filter(ys.rev, pos)
var s = 0
for r in 0..262144:
    let k = r % n
    s += first(ys) ?? 0 // timed: seq.first
    s += last(fr) ?? 0 // timed: seq.last
    s += (u: r, v: k).swap.0 // timed: seq.swap
    var one = [r]
    let (domain J, jv) = one.with_domain() // timed: seq.with_domain
    s += jv[J.first!]
    s += piece.covering(k / 16 * 16..n).len // timed: seq.covering
for r in 0..64:
    s += kv.buckets()[K.nth(0)].len // timed: seq.buckets
    s += Int(xs.cartesian_tree().root == None) // timed: seq.cartesian_tree
for r in 0..512:
    for p in ys.split(n / 2): // timed: seq.split
        s += p.len // timed: seq.split
    s += (0..n).pieces(cuts: cs).1.len // timed: seq.pieces
    s += ys.rekey(n..2 * n)[n] // timed: seq.rekey
for r in 0..16:
    s += zs.maximal(state: Counter<Int, Int>(), ok: c => c.len <= 2).len // timed: seq.maximal
    s += zs.ring.maximal(state: Counter<Int, Int>(), ok: c => c.len <= 2).len // timed: seq.Ring.maximal
    s += ws.minimal(state: track(0, (mut t, x) => t += x, (mut t, x) => t -= x), ok: w => w.state >= 6).len // timed: seq.minimal
    s += ws.ring.minimal(state: track(0, (mut t, x) => t += x, (mut t, x) => t -= x), ok: w => w.state >= 6).len // timed: seq.Ring.minimal
print(s)
