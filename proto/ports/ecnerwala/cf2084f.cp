@!multitest
use std.algebra.Monoid
use std.heap.MinHeap
use std.segtree.SegTree

let n: 2..=500_000 = read()
domain Pos = 0..n
domain Val = 1..=n
let a: [Pos]Val = read()
let c: [Pos]?Val = read(none: 0)

var fixed: [Val]?Pos = none
for i in Pos:
    if let v = c[i]:
        fixed[v] = i

var lo: [Val]Pos = Pos.first
var pref_max = SegTree<Val, Pos>(Monoid(Pos.first, max))
for i in Pos:
    let v = a[i]
    if let p = fixed[v]: pref_max.set(v, p)
    lo[v] = pref_max.fold(..=v)

var hi: [Val]Pos = Pos.last
var suff_min = SegTree<Val, Pos>(Monoid(Pos.last, min))
for i in Pos.rev:
    let v = a[i]
    if let p = fixed[v]: suff_min.set(v, p)
    hi[v] = suff_min.fold(v..)

var by_lo: [Pos]List<Val> = []
for v in Val:
    by_lo[lo[v]].push(v)

var pq = MinHeap<(Pos, Val)>()
var b: [Pos]Val = Val.first

fn solve() -> Bool:
    for i in Pos:
        for v in by_lo[i]:
            pq.push((hi[v], v))
        let (deadline, v) = pq.pop() ?? return false
        if deadline < i: return false
        if let want = c[i]:
            if v != want: return false
        b[i] = v
    true

if solve():
    print(b)
else:
    print(-1)
