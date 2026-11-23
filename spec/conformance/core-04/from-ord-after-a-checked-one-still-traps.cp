// expect: R0102 at 6
// Each conversion into a domain checks its own value: one in-range `C(i)` proves nothing about a later `C(j)`.
domain C = 0..3
let i = [1][0]
let a = C(i)
let j = [5][0]
let b = C(j)
print(a.ord, b.ord)
