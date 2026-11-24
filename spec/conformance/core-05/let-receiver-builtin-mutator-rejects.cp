// expect: E0403
// rule: 05.4.2
// The built-in `swap` changes its receiver, so a `let` list cannot call it.
let a = [1, 2]
a.swap(0, 1) // <- here
