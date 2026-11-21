// Codeforces 2084 F: Skyscape.
// A smaller element can only move left past larger ones, so every pair with
// u < v and pos_a[u] < pos_a[v] must keep pos_b[u] < pos_b[v]. Hence each
// value v is constrained to [lo[v], hi[v]] where lo[v] is the maximum fixed
// position among u <= v at or before v in a (defaulting to Pos.first), and
// hi[v] is the minimum fixed position among u >= v at or after v in a
// (defaulting to Pos.last). Earliest-deadline-first then assigns values to
// positions in Pos order.

@!multitest
domain Pos = read(2..=500_000)
domain Val = 1..=Pos.count
let a: [Pos]Val = read()
let c: [Pos]?Val = read(none: 0)
let fixed: [Val]?Pos = c.inverse()

fn sweep(order: Seq<Val>, edge: Pos, op: fn(Pos, Pos) -> Pos, below: Bool) -> [Val]Pos:
    var out: [Val]Pos = edge
    var st = SegTree<Val, Pos>(Monoid(edge, op))
    for v in order:
        if let p = fixed[v]: st.set(v, p)
        out[v] = if below: st.fold(..=v) else: st.fold(v..)
    out

let lo = sweep(a, Pos.first, max, below: true)
let hi = sweep(a.rev, Pos.last, min, below: false)

let by_lo = Val.group(by: v => lo[v])
var pq = MinHeap<(Pos, Val)>()
var b: [Pos]Val = Val.first

for i in Pos:
    for v in by_lo[i]: pq.push((hi[v], v))
    let (deadline, v) = pq.pop() ?? return print(-1)
    if deadline < i: return print(-1)
    if let want = c[i]:
        if v != want: return print(-1)
    b[i] = v

print(b)
