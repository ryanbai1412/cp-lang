// Source: CCO 2018 Day 2 P2 (Boring Lectures), https://dmoj.ca/problem/cco18p5
// Problem: N <= 1e6 qualities a_i <= 1e9 and a window length K. The value is the largest a_i + a_j
//   with i != j inside some K consecutive lectures, i.e. |i - j| < K. Print it for the initial
//   array and after each of Q <= 1e5 point assignments.
// Solution: the best pair is some p with its best partner within distance K - 1, counted at
//   whichever of the two is larger (ties by index). An arg-max SegTree of ?Peak finds p's best partner
//   on each side, and a max SegTree holds a_p + partner when p beats it, else 0. An update can only
//   change the entries of the position and of its old best partner on each side, so refresh
//   those three: O((N + Q) log N).
// original: corpus/submissions/4801867.cpp
// Bounds: n: 2..=1e6; k: 2..=n; q: 0..=1e5; a: 1..=1e9; upds.pos: 1..=n; upds.x: 1..=1e9
let n: 2..=1e6 = read()
let k: 2..=n = read()
let q: 0..=1e5 = read()
domain I = 1..=n
var a: [I]1..=1e9 = read()
let upds: [q](pos: I, x: 1..=1e9) = read()

// The larger lecture wins, the later one on ties.
type Peak = (val: Int, at: I)
fn better(x: ?Peak, y: ?Peak) -> ?Peak:
    if let p = x:
        if let r = y:
            if (r.val, r.at) > (p.val, p.at): return y
        return x
    y

var peaks = SegTree<I, ?Peak>(Monoid(None, better), [I] i => Some((val: a[i], at: i)))
var pairs = SegTree<I, Int>(max)  // a[i] + its best partner, where i beats that partner

fn left(i: I) -> ?Peak = peaks.fold(I.nth(max(0, i.ord - k + 1))..i)
fn right(i: I) -> ?Peak:
    if let next = i.next: peaks.fold(next..=I.nth(min(n - 1, i.ord + k - 1))) else: None

fn refresh(i: I):
    var paired = 0
    if let p = better(left(i), right(i)):
        if (a[i], i) > (p.val, p.at): paired = a[i] + p.val
    pairs.set(i, paired)

for i in I: refresh(i)
print(pairs.fold(I.all))
for u in upds:
    let partners = [left(u.pos), right(u.pos)]
    a[u.pos] = u.x
    peaks.set(u.pos, Some((val: u.x, at: u.pos)))
    refresh(u.pos)
    for o in partners:
        if let p = o: refresh(p.at)
    print(pairs.fold(I.all))
