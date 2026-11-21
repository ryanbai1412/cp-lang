// Codeforces 2084 F: Skyscape.
// A smaller element can only move left past larger ones, so every pair with
// u < v and pos_a[u] < pos_a[v] must keep pos_b[u] < pos_b[v]. Hence each
// value v is constrained to [lo[v], hi[v]] where lo[v] is the maximum fixed
// position among u <= v at or before v in a (defaulting to Pos.first), and
// hi[v] is the minimum fixed position among u >= v at or after v in a
// (defaulting to Pos.last). Earliest-deadline-first then assigns values to
// positions in Pos order.

@!multitest
use std.algebra.Monoid

let n: 2..=500_000 = read()
domain Pos = 0..n
domain Val = 1..=n
let a: [Pos]Val = read()
let c: [Pos]?Val = read(none: 0)

var fixed: [Val]?Pos = none
for i in Pos:
    if let v = c[i]: fixed[v] = i

fn sweep(forward: Bool, edge: Pos, op: fn(Pos, Pos) -> Pos) -> [Val]Pos:
    var out: [Val]Pos = edge
    var st = SegTree<Val, Pos>(Monoid(edge, op))
    for k in Pos:
        let i = if forward: k else: Pos(n - 1 - k.ord)
        let v = a[i]
        if let p = fixed[v]: st.set(v, p)
        out[v] = if forward: st.fold(..=v) else: st.fold(v..)
    out

let lo = sweep(true, Pos.first, max)
let hi = sweep(false, Pos.last, min)

var by_lo: [Pos]List<Val> = []
for v in Val: by_lo[lo[v]].push(v)

var pq = MinHeap<(Pos, Val)>()
var b: [Pos]Val = Val.first

fn solve() -> Bool:
    for i in Pos:
        for v in by_lo[i]: pq.push((hi[v], v))
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
