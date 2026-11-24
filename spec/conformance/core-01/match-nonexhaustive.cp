// expect: E0314
// rule: 01.7.3
enum Dir = L | R
let d = Dir.L
match d: L => print(1) // <- here
