// expect: R0201 at 3
// rule: 03.1.4 03.4.1
var a: [3]0..=9 = [3].filled(0)
var x = 5
a[0] = x * 2
