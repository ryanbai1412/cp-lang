// expect: E0902 at 2
// rule: 03.1.4
var a: [3]0..=9 = [3].filled(0)
a[0] = 10
