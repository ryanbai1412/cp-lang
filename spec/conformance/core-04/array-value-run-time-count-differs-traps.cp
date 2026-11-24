// expect: R0001
// rule: 04.8.10
// input: "5\n"
// An array value whose count is known only as the program runs is checked against its declared type then, in every build: a wrong count is a type mismatch, not a debug check.
let n: Int = read()
let d: [0..3]Int = [0..n] x => x // <- here
print(d.len)
