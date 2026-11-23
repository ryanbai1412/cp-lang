// expect: E0403 at 3
// rule: 05.4.2
// The built-in `swap` changes its receiver, so a `let` list cannot call it.
let a = [1, 2]
a.swap(0, 1)
