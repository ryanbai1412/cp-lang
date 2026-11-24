// expect: E0305
// rule: 07.4.7 07.5.3
// A reversed view walks out of key order, so it is not indexable; the fix writes `.nth(0)`.
let b = [5, 9, 1].sorted().rev
print(b[0]) // <- here
