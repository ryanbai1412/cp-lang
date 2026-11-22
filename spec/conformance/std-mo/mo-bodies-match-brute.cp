// expect: ok, output "2 1 0\n0 1 2\nYES\nYES\nYES"
use std.mo.{mo_order_hilbert, mo_rollback, mo_with_updates, mo}
use std.rand.{Rng}
domain P = 0..4
print(mo_order_hilbert([P(1)..P(1), P(0)..P(1), P(0)..P(0)]))
print(mo_order_hilbert([P(0)..P(2), P(1)..P(3), P(2)..P(2)]).sorted())
domain N = 0..40
var rng = Rng(7)
var a: [N]Int = 0
for i in N: a[i] = rng.int(0, 9)
var qs = List<Range<N>>()
for _ in 0..60:
    let l = rng.int(0, 39)
    let r = rng.int(l, 39)
    qs.push(N(l)..=N(r))
var s = 0
let got = mo_rollback(qs, mark: () => s, add: i => s += a[i], answer: _ => s, undo: m => s = m, reset: () => s = 0)
var ok = true
for qi in 0..qs.len:
    var t = 0
    for i in qs[qi]: t += a[i]
    if t != got[qi]: ok = false
print(ok)
let hord = mo_order_hilbert(qs)
print(hord.sorted() == [i for i in 0..qs.len])
var ups = List<(pos: N, by: Int)>()
for _ in 0..15: ups.push((pos: N(rng.int(0, 39)), by: rng.int(1, 5)))
var tq = List<(range: Range<N>, time: Int)>()
for k in 0..qs.len: tq.push((range: qs[k], time: rng.int(0, 15)))
var b: [N]Int = 0
for i in N: b[i] = a[i]
var w = 0
fn up(u: (pos: N, by: Int), win: Range<N>):
    b[u.pos] += u.by
    if u.pos in win: w += u.by
fn down(u: (pos: N, by: Int), win: Range<N>):
    b[u.pos] -= u.by
    if u.pos in win: w -= u.by
let got2 = mo_with_updates(tq, ups, add: i => w += b[i], remove: i => w -= b[i], apply_update: up, undo_update: down, answer: _ => w)
var ok2 = true
for qi in 0..tq.len:
    var c: [N]Int = 0
    for i in N: c[i] = a[i]
    for t in 0..tq[qi].time: c[ups[t].pos] += ups[t].by
    var t = 0
    for i in tq[qi].range: t += c[i]
    if t != got2[qi]: ok2 = false
print(ok2)
