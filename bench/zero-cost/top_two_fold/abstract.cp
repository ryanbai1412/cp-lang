// utso15p5's shape: per query, map a short run to (hi, lo) and fold it with a merge that
// keeps the two largest distinct values via a 4-element list. Neither the mapped run nor the
// 4-element list is built: the fold is sunk into the producer and the list is a fixed array.
type Top = (hi: Int, lo: Int)
fn both(a: Top, b: Top) -> Top:
    let hi = max(a.hi, b.hi)
    (hi, [x for x in [a.hi, a.lo, b.hi, b.lo] if x < hi].max() ?? 0)

let (n, q): (1..=200_000, 1..=2_000_000) = read()
let a: [0..n]1..=1_000_000 = read()
var total = 0
for k in 0..q:
    let l = k * 7919 % (n - 8)
    let t = (l..l + 8).map(i => (hi: a[i], lo: 0)).fold_from((hi: 0, lo: 0), both)
    total += t.hi + t.lo
print(total)
