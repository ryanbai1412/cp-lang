// expect: E0403
// rule: R05.4.2
// A method that changes its receiver lends it `mut`, which a `let` binding cannot be.
let a = [1, 2]
a.push(3) // <- here
print(a.len)
