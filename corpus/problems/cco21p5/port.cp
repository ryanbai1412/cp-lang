// CCO 2021 P5 (CP Lang 0.2.2): `LazySegTree<Slot, Int>(AddMin)` directly replaces
// the pre-0.2.2 negated `AddMax` + `INF` sentinel + `set_dp` workaround.
// original: corpus/submissions/4554642.cpp

let (n, m): (2..=200_000, 1..=200_000) = read()
domain Slot = 0..=n

var min_nb: [0..n]Int = n
for _ in 0..m:
    let (a, b): (1..=n, 1..=n) = read()
    let (u, v) = (min(a, b) - 1, max(a, b) - 1)
    min_nb[v] min= u

var at_min: [0..n]List<Int> = []
for i in 0..n:
    if min_nb[i] < n: at_min[min_nb[i]].push(i)

var suff: [0..=n]Int = n
for i in (0..n).rev: suff[i] = min(suff[i + 1], min_nb[i])

fn opt_min(a: ?Int, b: ?Int) -> ?Int:
    var out = a
    out min= b
    out

let add_min = Action<?Int, Int>(
    0,
    (f, g) => f + g,
    (f, x, _) => x + f,
    over: Monoid<?Int>(None, opt_min),
)
var tree = LazySegTree<Slot, ?Int, Int>(add_min)
tree.set(Slot.last, 0)

var j = n
var ans = 0
for i in (1..n).rev:
    while j >= 0 and suff[j] >= i: j -= 1
    for a in at_min[i]: tree.apply(Slot(a + 1).., 1)
    var cur = tree.fold(Slot(j + 1)..) ?? 0
    if min_nb[i] >= n:
        cur += 1
        tree.apply(Slot(i + 1).., 1)
    tree.set(Slot(i), cur)
    if i == 1: ans = cur

print(ans)
