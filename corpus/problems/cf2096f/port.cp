// Source: Codeforces 2096F (Wonderful Impostors), https://codeforces.com/contest/2096/problem/F
// limits: 2000 ms, 256 MB
// Problem: n viewers, each crewmate or impostor; m <= 2e5 statements, each "no impostor in
//   [a, b]" or "at least one impostor in [a, b]". q <= 2e5 questions l r: can statements l..r all
//   be true? Multitest, sums of n, m, q <= 2e5.
// Solution: statements fit iff every "some" interval has a viewer no "no" interval covers (make
//   those impostors). Fitting survives shrinking, so two pointers find the smallest fitting l for
//   each r and a question checks l >= first_fit[r]. A "some" interval fits iff its min cover is 0
//   (range-add min SegTree); a "no" interval fits iff no live "some" interval lies inside the
//   covered run it creates (left_while/right_while), via a min SegTree of the shortest live end
//   per start (a sliding Window each): O((m + q) log n).
// Bounds: n, m: 1..=200_000; st.a, st.b: 1..=n; ql, qr: 1..=m
@!multitest
let (n, m): (1..=200_000, 1..=200_000) = read()
domain V = 1..=n
domain S = 1..=m
let st: [S](some: Bool, a: V, b: V) = read()

var cover = SegTree<V, Int>(min, fill: 0, act: add)
var ends = [V] _ => Window(min)
var shortest = SegTree<V, ?Int>(min, fill: None)

fn fits(i: S) -> Bool:
    let s = st[i]
    if s.some: return cover.fold(s.a..=s.b) == 0
    let lo = cover.left_while(s.a.l, c => c > 0)
    let hi = cover.right_while(s.b.r, c => c > 0)
    shortest.fold(lo..hi).map(e => e >= hi.ord) ?? true

fn take(i: S):
    let s = st[i]
    if s.some:
        ends[s.a].push(s.b.ord)
        shortest.set(s.a, ends[s.a].all)
    else:
        cover.apply(s.a..=s.b, 1)

fn drop(i: S):
    let s = st[i]
    if s.some:
        ends[s.a].pop(s.b.ord)  // statements leave in the order they came
        shortest.set(s.a, ends[s.a].all)
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
