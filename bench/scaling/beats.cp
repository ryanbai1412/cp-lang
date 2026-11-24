// fixed: V
use std.beats.{BeatsSegTree, HistoricBeatsSegTree}
let n: Int = read()
domain D = 0..n
let reps = 1 << 16
let a: [D]Int = [D] i => (Int(i) * 7919) % 1000
fn span(i: Int) -> Range<D>:
    let l = (i * 7919) % n
    let r = l + 1 + (i * 104729) % (n - l)
    D(l)..=D(r - 1)
var s = 0
for _ in 0..200:
    s += BeatsSegTree(a).all_sum // timed: beats.BeatsSegTree.new
    var hn = HistoricBeatsSegTree(a) // timed: beats.HistoricBeatsSegTree.new
    s += hn.sum(D(0)..=D(n - 1))
var c = BeatsSegTree(a)
for i in 0..reps:
    c.chmin(span(i), 1000 - i % 997) // timed: beats.BeatsSegTree.chmin
    c.chmax(span(i + 1), i % 991) // timed: beats.BeatsSegTree.chmax
var t = BeatsSegTree(a)
for i in 0..reps:
    t.add(span(i), i % 7 - 3) // timed: beats.BeatsSegTree.add
    s += t.sum(span(i + 3)) // timed: beats.BeatsSegTree.sum
    s += t.min_val(span(i + 5)) ?? 0 // timed: beats.BeatsSegTree.min_val
    s += t.max_val(span(i + 7)) ?? 0 // timed: beats.BeatsSegTree.max_val
    s += t.get(D((i * 13) % n)) // timed: beats.BeatsSegTree.get
    t.set(D((i * 17) % n), i % 1000) // timed: beats.BeatsSegTree.set
    t.assign(span(i + 9), i % 1000) // timed: beats.BeatsSegTree.assign
for _ in 0..1 << 21:
    s += t.all_sum // timed: beats.BeatsSegTree.all_sum
    s += t.all_min ?? 0 // timed: beats.BeatsSegTree.all_min
    s += t.all_max ?? 0 // timed: beats.BeatsSegTree.all_max
var m = BeatsSegTree(a)
for i in 0..reps:
    m.chdiv(span(i), 2 + i % 3) // timed: beats.BeatsSegTree.chdiv
    m.chmod(span(i + 1), 7 + i % 50) // timed: beats.BeatsSegTree.chmod
    m.set(D((i * 17) % n), (i * 7919) % 1000)
var h = HistoricBeatsSegTree(a)
for i in 0..reps:
    h.chmin(span(i), 1000 - i % 997) // timed: beats.HistoricBeatsSegTree.chmin
    h.chmax(span(i + 1), i % 991) // timed: beats.HistoricBeatsSegTree.chmax
var g = HistoricBeatsSegTree(a)
for i in 0..reps:
    g.add(span(i), i % 7 - 3) // timed: beats.HistoricBeatsSegTree.add
    s += g.sum(span(i + 3)) // timed: beats.HistoricBeatsSegTree.sum
    s += g.max_val(span(i + 5)) ?? 0 // timed: beats.HistoricBeatsSegTree.max_val
    s += g.hist_max(span(i + 7)) ?? 0 // timed: beats.HistoricBeatsSegTree.hist_max
    s += g.hist_sum(span(i + 9)) // timed: beats.HistoricBeatsSegTree.hist_sum
print(s)
