// expect: E0902 at 1
// rule: 02.3.2
let s: Str of "01" = "012"
print(s.len)
