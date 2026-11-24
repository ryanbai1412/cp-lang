// expect: E0107
// `let mut` is a foreign spelling of `var`; the error names the cp spelling.
let mut x = 1 // <- here
x += 1
print(x)
