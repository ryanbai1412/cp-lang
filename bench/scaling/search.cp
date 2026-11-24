// fixed: eps
use std.search.{bsearch_real, ternary_max, ternary_min, ternary_max_real, ternary_min_real, parallel_bsearch, gallop_first, dinkelbach, split_best}
let n: Int = read()
let m = n / 16
let up = (x: F64) => x >= 0.3 // within: search.bsearch_real
let hill = (x: F64) => -(x - 1.0) * (x - 1.0) // within: search.ternary_max_real
let bowl = (x: F64) => (x - 2.0) * (x - 2.0) // within: search.ternary_min_real
let top = (x: F64) => -(x - 1.0) * (x - 1.0) // within: search.peak
let pit = (x: F64) => (x - 2.0) * (x - 2.0) // within: search.valley
var s = 0
var t = 0.0
for i in 0..512:
    if bsearch_real(0.0, 1.0, up, steps: m) != None: s += 1 // timed: search.bsearch_real
    t += ternary_max_real(0.0, 4.0, hill, steps: m) // timed: search.ternary_max_real
    t += ternary_min_real(0.0, 4.0, bowl, steps: m) // timed: search.ternary_min_real
    t += (0.0..4.0).peak(by: top, steps: m) // timed: search.peak
    t += (0.0..4.0).valley(by: pit, steps: m) // timed: search.valley
for i in 0..2 * n:
    let f = (j: Int) => -(j - i) * (j - i) // within: search.ternary_max
    let g = (j: Int) => (j - i) * (j - i) // within: search.ternary_min
    s += ternary_max(0..n, f) ?? 0 // timed: search.ternary_max
    s += ternary_min(0..n, g) ?? 0 // timed: search.ternary_min
for i in 0..8 * n:
    let ok = (j: Int) => j >= n - i % 7 // within: search.gallop_first
    s += gallop_first(0, ok) ?? 0 // timed: search.gallop_first
let solve = (l: F64) => if l < 1.0: (num: 3.0, den: 2.0) else: (num: 2.0, den: 1.0) // within: search.dinkelbach
for i in 0..8 * n:
    t += dinkelbach(0.0, solve: solve) // timed: search.dinkelbach
var c = 0
let reset = () => c = 0 // within: search.parallel_bsearch
let apply = (_: Int) => c += 1 // within: search.parallel_bsearch
let past = (q: Int) => c > q // within: search.parallel_bsearch
for i in 0..8:
    let ans = parallel_bsearch(n, 0..n, reset: reset, apply: apply, ok: past) // timed: search.parallel_bsearch
    s += ans[i] ?? 0
let a = [i * 7919 % n for i in 0..n]
let whole = (g: Range<Int>) => g // within: search.split_best
let first = (_: Range<Int>, r: Range<Int>) => (a[r.start], r.start) // within: search.split_best
for i in 0..8:
    let xs = split_best([0..n], range: whole, best: first).collect() // timed: search.split_best
    s += xs[i]
if t > 0.0: s += 1
print(s)
