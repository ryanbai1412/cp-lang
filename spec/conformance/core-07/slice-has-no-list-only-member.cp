// expect: E0401
// rule: 07.4.17
// tool: check
// has: ".to_list()"
// `push` grows a `List`; a slice is a fixed-length view of its source, so the error points at copying it into a list first.
let xs = [1, 2, 3, 4]
let s = xs[1..3]
s.push(9) // <- here
