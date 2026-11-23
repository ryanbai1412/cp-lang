// expect: E0305 at 3
// rule: 07.4.5
// A literal that is not a key of the view is an error naming `.nth(0)` (decision 388).
let s = [10, 11, 12, 13, 14, 15]
print(s[2..5][0])
