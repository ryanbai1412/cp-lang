// expect: E0302
// rule: R04.4.6
// A closed range of cuts names no piece, so it can't index the positions between them.
domain V = 1..=4
let a = [V] v => Int(v)
print(a[V.cuts(1)..=V.cuts(3)].len) // <- here
