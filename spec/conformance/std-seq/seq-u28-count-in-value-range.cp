// expect: ok, output "1 3 3 0 0 2"
// Acceptance for seq/U28 (how many sorted values fall in `[lo, hi)`), handbook/product/std/sequences-usecases.md.
let a = [1, 3, 3, 5, 8]
let e = a.equal_range(3)
print(e.l, e.r, a.count_in(3..6), a.count_in(4..5), a.count_in(6..2), [(1, 'a'), (4, 'b'), (4, 'c')].count_in(4..9, by: p => p.0))
