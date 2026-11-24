// expect: R0101
// rule: 04b.1.5 04b.4.2
let s = [4, 5, 6, 7]
var t = [s[1..3].keys]0
let i = 0
t[i] = 1 // <- here
