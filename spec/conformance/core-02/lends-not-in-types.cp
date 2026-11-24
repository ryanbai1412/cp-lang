// expect: E0105
// rule: 02.1.4
// Owned or lent is not part of a type: `lends` has no place in an annotation.
domain V = 0..3
let a: [V]Int = 0
let w: lends [V]Int = a[..] // <- here
print(w.len)
