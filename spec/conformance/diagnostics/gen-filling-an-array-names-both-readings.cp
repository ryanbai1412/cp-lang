// expect: E0501
// rule: 06.4.3
// has: "gen<[0..3]0..=9>()"
// has: "gen<0..=9>()"
// A bare `gen()` filling a declared array reads two ways, a random array or one
// random value in every cell, so it stays an error and the help spells both.
let v: [0..3]0..=9 = gen() // <- here
print(v[0])
