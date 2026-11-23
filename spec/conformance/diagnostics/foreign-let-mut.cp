// expect: E0107 at 2
// `let mut` is a foreign spelling of `var`; the error names the cp spelling.
let mut x = 1
x += 1
print(x)
