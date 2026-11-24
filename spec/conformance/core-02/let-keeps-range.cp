// expect: E0902
// rule: 02.10.2
var x: 0..=9 = 0
x = 5
let y = 7
x = y + 5 // <- here
