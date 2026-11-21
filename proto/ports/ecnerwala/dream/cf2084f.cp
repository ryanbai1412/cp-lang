// cf2084f: a smaller element can only move left past larger ones, so every pair
// with u < v and pos_a[u] < pos_a[v] must keep pos_b[u] < pos_b[v].
// Two sweeps compute each value's allowed position window [lo[v], hi[v]];
// then earliest-deadline-first matches values to positions.
--- with multitest() ---

domain Pos = read(2..=5e5)
domain Val = 1..=Pos.count
let a: [Pos]Val = read()
let c: [Pos]?Val = read(none: 0)
let fixed: [Val]?Pos = c.invert()

// Forward sweep: lo[v] = max fixed position among u <= v appearing at or before v in a.
var lo: [Val]Pos = Pos.first
var pref_max = SegTree<Val, Max<Pos>>(init: none)
for v in a:
    if let p = fixed[v]: pref_max.update(v, p)
    lo[v] = pref_max.fold(..=v) ?? Pos.first

// Backward sweep: hi[v] = min fixed position among u >= v appearing at or after v in a.
var hi: [Val]Pos = Pos.last
var suff_min = SegTree<Val, Min<Pos>>(init: none)
for v in a.rev:
    if let p = fixed[v]: suff_min.update(v, p)
    hi[v] = suff_min.fold(v..) ?? Pos.last

let by_lo = Val.group(by: v => lo[v])
var pq = MinHeap<(Pos, Val)>()
var b: [Pos]Val = Val.first

for i in Pos:
    for v in by_lo[i]: pq.push((hi[v], v))
    let (deadline, v) = pq.pop() ?? return print(-1)
    if deadline < i or (c[i] != none and c[i] != v):
        return print(-1)
    b[i] = v

print(b)
