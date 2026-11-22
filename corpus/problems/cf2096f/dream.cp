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
use std.window.{Window}
@!multitest
let (n, m): (1..=200_000, 1..=200_000) = read()
domain V = 1..=n
domain S = 1..=m
let st: [S](some: Bool, a: V, b: V) = read()

var cover = SegTree<V, Int>(min, fill: 0, act: add)
var ends: [V]Window<V.cuts> = Window(min)   // pieces leave in the order they came
var shortest = SegTree<V, ?V.cuts>(min, fill: none)

fn fits(i: S) -> Bool:
    let s = st[i]
    if s.some: return cover.fold(s.a..=s.b) == 0
    let lo = cover.left_while(to: s.a.l, ok: c => c > 0)
    let hi = cover.right_while(from: s.b.r, ok: c => c > 0)
    shortest.fold(lo..hi).is_none_or(e => e > hi)

fn take(i: S):
    let s = st[i]
    if s.some:
        ends[s.a].push(s.b.r)
        shortest[s.a] = ends[s.a].all
    else:
        cover.apply(s.a..=s.b, 1)

fn drop(i: S):
    let s = st[i]
    if s.some:
        ends[s.a].pop()
        shortest[s.a] = ends[s.a].all
    else:
        cover.apply(s.a..=s.b, -1)

var first_fit: [S]S = S.first!
var l = S.first!
for r in S:
    while not fits(r):
        drop(l)
        l = l.next!
    take(r)
    first_fit[r] = l

let q: Int = read()
for _ in 0..q:
    let (ql, qr): (S, S) = read()
    print(yes(ql >= first_fit[qr]))
