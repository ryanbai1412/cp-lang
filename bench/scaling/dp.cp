use std.dp.{Line, LiChao, LineContainer, MongePath, MonotoneHull, monotone_minima, dnc_layer, knuth, aliens, subset_sums, monge_path, monge_paths, smawk, lis_ends, supermasks, masks_of_size, subset_folds, backtrack}
let n: Int = read()
let reps = 1 << 17
var s = 0
let xs = [i * 7919 % n for i in 0..n]
var lg = 0
while (1 << (lg + 1)) <= n: lg += 1
var rt = 0
while (rt + 1) * (rt + 1) <= n: rt += 1

for i in 0..reps:
    s += Line(i % 7, i % 5).at(i % n) // timed: dp.Line.at
for _ in 0..reps:
    var e = LiChao(min, over: 0..=n) // timed: dp.LiChao.new
    s += 1
var lc = LiChao(min, over: 0..=n)
for i in 0..reps:
    lc.add(Line(i % 101 - 50, xs[i % n])) // timed: dp.LiChao.add
    s += lc.at(i * 13 % n) ?? 0 // timed: dp.LiChao.at
    s += (lc.line_at(i * 17 % n) ?? Line(0, 0)).k // timed: dp.LiChao.line_at
for i in 0..reps:
    var c = LiChao(min, over: 0..=n)
    c.add(Line(1, i))
    c.clear() // timed: dp.LiChao.clear

var hull = LineContainer(max)
for i in 0..reps:
    hull.add(Line(xs[i % n] - n / 2, xs[i * 3 % n])) // timed: dp.LineContainer.add
    s += hull.at(i * 13 % n) ?? 0 // timed: dp.LineContainer.at
    s += (hull.line_at(i * 17 % n) ?? Line(0, 0)).m // timed: dp.LineContainer.line_at
    s += hull.len // timed: dp.LineContainer.len
    if hull.is_empty: s += 1 // timed: dp.LineContainer.is_empty
var full = LineContainer(max)
for i in 0..n: full.add(Line(i, -i * i))
for _ in 0..40:
    s += full.each().count() // timed: dp.LineContainer.each

for _ in 0..40:
    var h = MonotoneHull<()>(min)
    for i in 0..n:
        h.add(Line(n - i, i * i % 1000)) // timed: dp.MonotoneHull.add
    for i in 0..n:
        s += h.at_next(i) ?? 0 // timed: dp.MonotoneHull.at_next
var mh = MonotoneHull<Int>(min)
// tangents of -x^2 at 0..n: every line is best somewhere, so all n stay
for i in 0..n: mh.add(Line(-2 * i, i * i), tag: i)
for i in 0..reps:
    s += mh.at(i * 7919 % n) ?? 0 // timed: dp.MonotoneHull.at
    s += mh.len // timed: dp.MonotoneHull.len
    if mh.is_empty: s += 1 // timed: dp.MonotoneHull.is_empty
for _ in 0..500:
    s += mh.items.count() // timed: dp.MonotoneHull.items
for i in 0..reps:
    var h = MonotoneHull<()>(min)
    h.add(Line(1, i))
    h.clear() // timed: dp.MonotoneHull.clear
for _ in 0..40:
    var h = MonotoneHull<Int>(min)
    for i in 0..n: h.add(Line(n - i, i), tag: i)
    for i in 0..n:
        s += (h.best_next(i) ?? (val: 0, tag: 0)).tag // timed: dp.MonotoneHull.best_next

let sq_smawk: (Int, Int) -> Int = (a, b) => (b - a) * (b - a) + 7 // within: dp.smawk
let sq_path: (Int, Int) -> Int = (a, b) => (b - a) * (b - a) + 7 // within: dp.monge_path
let sq_paths: (Int, Int) -> Int = (a, b) => (b - a) * (b - a) + 7 // within: dp.monge_paths
let sq_dnc: (Int, Int) -> Int = (a, b) => (b - a) * (b - a) + 7 // within: dp.dnc_layer
let sq_next: (Int, Int) -> Int = (a, b) => (b - a) * (b - a) + 7 // within: dp.MongePath.next
let sqo: (Int, Int) -> ?Int = (i, j) => Some((j - i) * (j - i)) // within: dp.monotone_minima
for _ in 0..8:
    s += monotone_minima(n, n, sqo).len // timed: dp.monotone_minima
    s += smawk(n, n, sq_smawk).len // timed: dp.smawk
    s += monge_path(xs.sorted(), sq_path).val // timed: dp.monge_path
    s += monge_paths(xs.sorted(), sq_paths).val.len // timed: dp.monge_paths
let prev = [if i == 0: Some(0) else: None for i in 0..n]
for _ in 0..8:
    s += dnc_layer(prev, sq_dnc).len // timed: dp.dnc_layer
for _ in 0..8:
    var p = MongePath(xs.sorted(), sq_next)
    for _ in 0..n - 1:
        s += p.next().prev // timed: dp.MongePath.next
var mp = MongePath(xs.sorted(), sq_path)
for _ in 0..n - 1: s += mp.next().prev
for i in 0..reps:
    s += mp.val(i % n) // timed: dp.MongePath.val
    s += mp.len // timed: dp.MongePath.len
    if mp.is_empty: s += 1 // timed: dp.MongePath.is_empty
for _ in 0..200:
    s += mp.items.count() // timed: dp.MongePath.items

let span: (Int, Int) -> Int = (i, j) => j - i // within: dp.knuth
for _ in 0..8:
    s += knuth(rt, span)[0][rt] // timed: dp.knuth [let: n = \sqrt n]
let pen: Int -> (val: Int, count: Int) = lam => (val: lam, count: if lam < 0: 2 else: 0) // within: dp.aliens
for i in 0..reps:
    s += aliens(1, pen, lambdas: -n..n) // timed: dp.aliens
let bits = [i % 5 + 1 for i in 0..lg]
for _ in 0..200:
    s += subset_sums(bits).len // timed: dp.subset_sums [let: n = log_2 n]
    s += subset_folds(bits, +).len // timed: dp.subset_folds [let: n = log_2 n]
    s += supermasks(0, lg).count() // timed: dp.supermasks [let: bits = log_2 n; m = 0]
for _ in 0..4000:
    s += masks_of_size(lg, 3).count() // timed: dp.masks_of_size [let: bits = log_2 n] [fixed: k]
for _ in 0..40:
    s += xs.lis_ends().len // timed: dp.lis_ends
fn kids(x: Int) -> yields Int:
    if x < n: // within: dp.backtrack
        yield 2 * x + 1 // within: dp.backtrack
        yield 2 * x + 2 // within: dp.backtrack
for _ in 0..40:
    s += backtrack(0, kids).count() // timed: dp.backtrack [let: visited = 2 n]
for _ in 0..1 << 15:
    for st in backtrack(0, kids):
        if st.depth == 1: st.prune() // timed: dp.Step.prune
print(s)
