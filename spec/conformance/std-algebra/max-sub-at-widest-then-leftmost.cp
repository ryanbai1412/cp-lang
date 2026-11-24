// expect: ok, output "1 0 3 1 0 3\n2 0 1\nYES"
// rule: std.algebra.MaxSubAt
// `MaxSubAt` finds the best subarray and where it is; ties go to the widest, then the leftmost.
// In [1, -1, 1] the widest best run is all three and it crosses the split after the second
// element, so the whole fold and the two halves joined must agree. Then random arrays against
// brute force, folding whole and split at every cut.
use std.algebra.{MaxSubAt}
domain D = 0..3
let a: [D]Int = [1, -1, 1]
let whole = a.items.map((i, v) => MaxSubAt.of(i, v)).fold(+)
let halves = (MaxSubAt.of(D(0), 1) + MaxSubAt.of(D(1), -1)) + MaxSubAt.of(D(2), 1)
print(whole.best, whole.at.l.ord, whole.at.r.ord, halves.best, halves.at.l.ord, halves.at.r.ord)
let b: [D]Int = [2, -5, 2]
let t = b.items.map((i, v) => MaxSubAt.of(i, v)).fold(+)
print(t.best, t.at.l.ord, t.at.r.ord)

domain E = 0..7
var ok = true
var seed = 7
for round in 0..300:
    var xs: [E]Int = 0
    for i in E:
        seed = (seed * 1103515245 + 12345) % 2147483648
        xs[i] = seed / 65536 % 5 - 2
    var key = (Int.MIN, 0, 0)
    var at = (0, 0)
    for l in 0..7:
        var s = 0
        for r in l + 1..=7:
            s += xs[E.nth(r - 1)]
            if (s, r - l, -l) > key: (key, at) = ((s, r - l, -l), (l, r))
    for c in 0..=7:
        let lhs = (0..c).map(k => MaxSubAt.of(E.nth(k), xs[E.nth(k)])).fold(+)
        let rhs = (c..7).map(k => MaxSubAt.of(E.nth(k), xs[E.nth(k)])).fold(+)
        let got = lhs + rhs
        if (got.best, got.at.l.ord, got.at.r.ord) != (key.0, at.0, at.1): ok = false
print(yes(ok))
