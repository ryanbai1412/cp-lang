// Codeforces 1146 H: Satanic Panic, tuned.
// port.cp's count, with the coordinates read as the statement bounds them
// (|x|, |y| <= 10^9), so the orientation test is a 64-bit cross product as in
// orig.cpp; std's `turn` takes unbounded `Pt`s and works in 128 bits. The
// bitsets are filled in place, not built from a list of members.
use std.math.{choose}

let n: 5..=300 = read()
domain V = 0..n
type C = -1_000_000_000..=1_000_000_000
let pts: [V](x: C, y: C) = read()

fn ccw(a: V, b: V, c: V) -> Bool:
    let (p, q, r) = (pts[a], pts[b], pts[c])
    (q.x - p.x) * (r.y - p.y) - (q.y - p.y) * (r.x - p.x) > 0

var left: [V, V]Bitset<V> = Bitset<V>()
for i in V:
    for j in V:
        for k in V if ccw(i, j, k): left[i, j].add(k)

// A triangle with `inside` points inside it adds C(inside, 2) - inside * (n - 4) / 2 to c5.
// That is a half-integer for some triangles, so sum twice the excess and halve once at the end.
fn twice_excess(inside: Int) -> Int = inside * (inside - 1) - inside * (n - 4)
let twice_total = sum(
    twice_excess(
        (left[i, j] & left[j, k] & left[k, i]).len
    ) for i in V for j in V.after(i) for k in V.after(i) if k in left[i, j]
)
print(choose(n, 5) + twice_total / 2)
