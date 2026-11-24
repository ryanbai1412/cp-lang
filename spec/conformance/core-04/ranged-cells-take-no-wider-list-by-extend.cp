// expect: E0302
// rule: 04.4.24
// `extend` copies each of `zs`'s cells into `ys`'s `0..100` cells, so `zs` must be no wider.
var ys: List<0..100> = [1]
let zs: List<Int> = [500]
ys.extend(zs) // <- here
ys.swap(0, 1)
print(ys[0])
