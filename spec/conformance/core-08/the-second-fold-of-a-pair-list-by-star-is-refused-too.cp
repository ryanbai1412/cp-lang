// expect: E0606
// rule: 08.2.2
// Each call of `fold(*)` on a pair list fails `*`'s `Op` bound on its own; the second call is refused as well as the first.
let ps = [(1, 2), (3, 4)]
print(ps.fold(*))
print(ps.fold(*)) // <- here
