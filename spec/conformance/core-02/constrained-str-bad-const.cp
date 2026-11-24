// expect: E0902
// rule: 02.3.2
let s: Str of "01" = "012" // <- here
print(s.len)
