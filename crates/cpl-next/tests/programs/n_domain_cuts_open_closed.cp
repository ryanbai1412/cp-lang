// expect: E0302
// rule: R04.4.6
// `..=c` with `c` a cut names no piece of positions.
domain V = 1..=5
let a = [V] v => Int(v)
print(a[..=V.cuts(2)].sum()) // <- here
