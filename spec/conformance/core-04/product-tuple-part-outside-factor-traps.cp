// expect: R0102 at 7
// rule: 04.2.10
// `P((r, c))` checks each integer part into its factor, as `R(2)` does: `(2, 0)` has its first part outside `R`.
domain R = 0..2
domain C = 0..3
domain D = (R, C)
let p = D((2, 0))
print(p.0, p.1)
