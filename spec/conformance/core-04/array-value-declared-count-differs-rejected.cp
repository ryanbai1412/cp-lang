// expect: E0302
// rule: 04.8.10
// An array value whose count differs from the one its declared type writes is a type mismatch, as a list literal's is: `[0..=3]` has 4 keys, `[0..3]` has 3.
let a: [0..3]Int = [0..=3] x => x // <- here
print(a.len)
print(a[3])
