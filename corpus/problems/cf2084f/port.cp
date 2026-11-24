// Source: Codeforces 2084F (Skyscape), https://codeforces.com/contest/2084/problem/F
// limits: 4000 ms, 500 MB
// Problem: a permutation a and a partial permutation c (0 = missing) of length n. b is good if
//   at most n operations "pick l < r with a_r = min(a_l..a_r), rotate a_l..a_r right by one" turn
//   a into b. Print a good b that extends c, or -1. n <= 5e5, sum of n <= 5e5; multitest.
// Solution: a smaller value can only move left past larger ones, so u < v with u before v in a
//   keep that order in b. v must sit in [lo[v], hi[v]]: lo is the largest fixed position of a
//   u <= v at or before v in a, hi the smallest of a u >= v at or after v (sweeps with max/min
//   SegTrees over values); a fixed v needs both at its own cell. Earliest deadline first fills
//   the free positions in order from a MinHeap of (hi, v) over the free values, failing on a
//   missed deadline; ties go to the smaller value, which keeps the order. O(n log n).
// Bounds: a: 1..=Pos.count

@!multitest
use std.seq.{buckets}

domain Pos = read(2..=500_000)
domain Val = 1..=Pos.count
let a: [Pos]Val = read()
let c: [Pos]?Val = read(none: 0)
let fixed: [Val]?Pos = c.inverse()

fn sweep<S: Seq>(order: S, edge: Pos, op: fn(Pos, Pos) -> Pos, below: Bool) -> [Val]Pos:
    var out: [Val]Pos = edge
    var st = SegTree<Val, Pos>(Monoid(edge, op))
    for v in order:
        if let p = fixed[v]: st.set(v, p)
        out[v] = if below: st.fold(..=v) else: st.fold(v..)
    out

let lo = sweep(a, Pos.first!, max, below: true)
let hi = sweep(a.rev, Pos.last!, min, below: false)

let by_lo = buckets(lo)
var pq = MinHeap<(Pos, Val)>()
var b: [Pos]Val = Val.first!

for i in Pos:
    for v in by_lo[i]:
        if fixed[v] == None: pq.push((hi[v], v))
    if let want = c[i]:
        if lo[want] != i or hi[want] != i: return print(-1)
        b[i] = want
        continue
    let (deadline, v) = pq.pop() ?? return print(-1)
    if deadline < i: return print(-1)
    b[i] = v

print(b)
