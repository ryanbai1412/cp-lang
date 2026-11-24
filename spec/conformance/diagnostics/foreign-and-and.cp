// expect: E0107
// `&&` is a foreign spelling of `and`.
let a = true
let b = a && a // <- here
print(b)
