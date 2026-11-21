// Codeforces 2096 F: Wonderful Impostors.
// Statements l..r can all be true iff every "some impostor" interval keeps a viewer
// no "no impostor" interval covers: then make every uncovered viewer an impostor.
// That only gets easier as the window shrinks, so two pointers give, for each r, the
// smallest l that still fits, and a query l r asks whether l reaches it.
// - A "no impostor" interval [a, b] merges the covered run around it into the gap
//   between the nearest uncovered viewers; it fits iff no live "some impostor"
//   interval lies inside that gap.
// - A "some impostor" interval fits iff it has an uncovered viewer.
// `cover` counts the live "no impostor" intervals over each viewer (min 0 means one
// is uncovered). The live "some impostor" intervals starting at v are a sliding
// window over the statements, so `ends[v]` keeps their shortest right end, and
// `shortest` is a min tree over those.
use std.window.{MonoDeque}
@!multitest
let (n, m): (1..=200_000, 1..=200_000) = read()
domain V = 1..=n
let st: [0..m](some: Bool, a: V, b: V) = read()

var cover = LazySegTree<V, Int>(AddMin, fill: 0)
var ends: [V]MonoDeque<Int> = MonoDeque()
var shortest = SegTree<V, Int>(min, fill: n + 1)

fn fits(i: Int) -> Bool:
    let s = st[i]
    if s.some: return cover.fold(s.a..=s.b) == 0
    let lo = cover.min_left(to: s.a.l, ok: c => c > 0)
    let hi = cover.max_right(from: s.b.r, ok: c => c > 0)
    lo == hi or shortest.fold(lo..hi) >= hi.ord

fn take(i: Int):
    let s = st[i]
    if s.some:
        ends[s.a].push(at: i, val: s.b.ord)
        shortest.set(s.a, ends[s.a].best!)
    else:
        cover.apply(s.a..=s.b, 1)

fn drop(i: Int):
    let s = st[i]
    if s.some:
        ends[s.a].drop_before(i + 1)
        shortest.set(s.a, ends[s.a].best ?? (n + 1))
    else:
        cover.apply(s.a..=s.b, -1)

var first_fit: [0..m]Int = 0
var l = 0
for r in 0..m:
    while not fits(r):
        drop(l)
        l += 1
    take(r)
    first_fit[r] = l

let q: Int = read()
for _ in 0..q:
    let (ql, qr): (1..=m, 1..=m) = read()
    print(if ql - 1 >= first_fit[qr - 1]: "YES" else: "NO")
