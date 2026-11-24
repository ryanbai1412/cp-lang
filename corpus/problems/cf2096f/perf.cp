// Tuned: the cover counts sit in a plain lazy-count tree, as orig.cpp's (a node's count is
// never pushed down; `full` marks a fully covered subtree), and a "no" interval finds the
// open viewers around it by descending that tree, instead of a range-add min SegTree's
// left_while/right_while.
//
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

var size = 1
while size < n: size *= 2
var count: [2 * size]Int = 0
var full: [2 * size]Bool = false
var ends = [V] _ => Window(min)
var shortest = SegTree<V, ?Int>(min, fill: None)

fn settle(p: Int): full[p] = count[p] > 0 or (p < size and full[2 * p] and full[2 * p + 1])

fn cover(a: V, b: V, by: Int):
    var (x, y) = (a.ord + size, b.ord + 1 + size)
    while x < y:
        if x % 2 == 1:
            count[x] += by
            settle(x)
            x += 1
        if y % 2 == 1:
            y -= 1
            count[y] += by
            settle(y)
        (x, y) = (x / 2, y / 2)
    for leaf in [a.ord + size, b.ord + size]:
        var p = leaf / 2
        while p > 0:
            settle(p)
            p /= 2

fn open_in(p: Int, x: Int, y: Int, a: Int, b: Int) -> Bool:
    if full[p] or b <= x or y <= a: return false
    if a <= x and y <= b: return true
    let mid = (x + y) / 2
    open_in(2 * p, x, mid, a, b) or open_in(2 * p + 1, mid, y, a, b)

fn last_open_before(p: Int, x: Int, y: Int, a: Int) -> ?Int:
    if full[p] or a <= x: return None
    if y - x == 1: return x
    let mid = (x + y) / 2
    last_open_before(2 * p + 1, mid, y, a) ?? last_open_before(2 * p, x, mid, a)

fn first_open_from(p: Int, x: Int, y: Int, a: Int) -> ?Int:
    if full[p] or y <= a: return None
    if y - x == 1: return x
    let mid = (x + y) / 2
    first_open_from(2 * p, x, mid, a) ?? first_open_from(2 * p + 1, mid, y, a)

fn fits(i: S) -> Bool:
    let s = st[i]
    if s.some: return open_in(1, 0, size, s.a.ord, s.b.ord + 1)
    let lo = V.cuts((last_open_before(1, 0, size, s.a.ord) ?? -1) + 1)
    let hi = V.cuts(min(first_open_from(1, 0, size, s.b.ord + 1) ?? n, n))
    shortest.fold(lo..hi).map(e => e >= hi.ord) ?? true

fn take(i: S):
    let s = st[i]
    if s.some:
        ends[s.a].push(s.b.ord)
        shortest.set(s.a, ends[s.a].all)
    else:
        cover(s.a, s.b, 1)

fn drop(i: S):
    let s = st[i]
    if s.some:
        ends[s.a].pop(s.b.ord)  // statements leave in the order they came
        shortest.set(s.a, ends[s.a].all)
    else:
        cover(s.a, s.b, -1)

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
