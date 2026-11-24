// expect: ok, output "ok"
// input: "1 300\n"
// model: AugSortedBag
// model: AugSortedSet
// AugSortedBag and AugSortedSet folding List concatenation (associative, not commutative, so
// a fold out of order shows) against a sorted List<Int>: seeded random add and remove over keys
// -8..=8, with len, all, count, contains, rank, nth, first, last, each and every fold compared
// after each op. Reads `seed n`; prints the first op that differs.
use std.rand.{Rng}
use std.bag.{AugSortedBag, AugSortedSet}

fn opt(a: List<Int>, i: Int) -> ?Int = if 0 <= i and i < a.len: Some(a[i]) else: None

fn cnt(a: List<Int>, x: Int) -> Int = [y for y in a if y == x].len

fn drop1(a: List<Int>, x: Int) -> List<Int>:
    var out: List<Int> = []
    var done = false
    for y in a:
        if y == x and not done:
            done = true
        else:
            out.push(y)
    out

fn folds(b: AugSortedBag<Int, List<Int>>, lo: Int, hi: Int) -> List<List<Int>> =
    [b.all, b.fold_lt(hi), b.fold_le(hi), b.fold_ge(lo), b.fold_gt(lo), b.fold(lo..hi), b.fold(lo..=hi), b.fold_between(lo, hi)]

fn set_folds(s: AugSortedSet<Int, List<Int>>, lo: Int, hi: Int) -> List<List<Int>> =
    [s.all, s.fold_lt(hi), s.fold_le(hi), s.fold_ge(lo), s.fold_gt(lo), s.fold(lo..hi), s.fold(lo..=hi), s.fold_between(lo, hi)]

fn want_folds(a: List<Int>, lo: Int, hi: Int) -> List<List<Int>> =
    [a, [y for y in a if y < hi], [y for y in a if y <= hi], [y for y in a if y >= lo], [y for y in a if y > lo],
     [y for y in a if lo <= y and y < hi], [y for y in a if lo <= y and y <= hi], [y for y in a if lo <= y and y < hi]]

let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
var b: AugSortedBag<Int, List<Int>> = AugSortedBag((x, y) => x + y, [], k => [k])
var s: AugSortedSet<Int, List<Int>> = AugSortedSet((x, y) => x + y, [], k => [k])
var a: List<Int> = []
var d: List<Int> = []
var bad = ""
for k in 0..n:
    let op = rng.int(0, 6)
    let x = rng.int(-8, 8)
    if op <= 2:
        b.add(x)
        a.push(x)
        a = a.sorted()
        if s.add(x) == (cnt(d, x) > 0): bad = "op {k}: set add {x}"
        if cnt(d, x) == 0:
            d.push(x)
            d = d.sorted()
    elif op <= 4:
        if b.remove(x) != (cnt(a, x) > 0): bad = "op {k}: remove {x}"
        a = drop1(a, x)
        if s.remove(x) != (cnt(d, x) > 0): bad = "op {k}: set remove {x}"
        d = drop1(d, x)
    elif rng.int(0, 3) == 0:
        b = AugSortedBag((x, y) => x + y, [], k => [k])
        s = AugSortedSet((x, y) => x + y, [], k => [k])
        a = []
        d = []
    else:
        b.reserve(rng.int(0, 40))
        s.reserve(rng.int(0, 40))
    if bad != "": break
    let q = rng.int(-9, 9)
    let hi = rng.int(-9, 9)
    let i = rng.int(-1, a.len + 1)
    if b.len != a.len or b.is_empty != (a.len == 0) or b.each().collect() != a:
        bad = "op {k}: bag {b.each().collect()} want {a}"
    elif b.first != opt(a, 0) or b.last != opt(a, a.len - 1):
        bad = "op {k}: bag first or last"
    elif b.count(q) != cnt(a, q) or b.rank(q) != [y for y in a if y < q].len or b.nth(i) != opt(a, i):
        bad = "op {k}: bag count, rank {q} or nth {i}"
    elif folds(b, q, hi) != want_folds(a, q, hi):
        bad = "op {k}: bag folds {q} {hi}: {folds(b, q, hi)}"
    elif s.len != d.len or s.is_empty != (d.len == 0) or s.each().collect() != d:
        bad = "op {k}: set {s.each().collect()} want {d}"
    elif s.first != opt(d, 0) or s.last != opt(d, d.len - 1) or s.contains(q) != (cnt(d, q) > 0):
        bad = "op {k}: set first, last or contains {q}"
    elif s.rank(q) != [y for y in d if y < q].len or s.nth(i) != opt(d, i):
        bad = "op {k}: set rank {q} or nth {i}"
    elif set_folds(s, q, hi) != want_folds(d, q, hi):
        bad = "op {k}: set folds {q} {hi}: {set_folds(s, q, hi)}"
    if bad != "":
        break
print(if bad == "": "ok" else: bad)
