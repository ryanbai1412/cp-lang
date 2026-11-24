// expect: E0107
// `||` is a foreign spelling of `or`.
let a = true
let b = a || a // <- here
print(b)
