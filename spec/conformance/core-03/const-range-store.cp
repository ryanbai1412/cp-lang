// expect: E0902
// rule: 03.1.4
var a: [3]0..=9 = [3].filled(0)
a[0] = 10 // <- here
