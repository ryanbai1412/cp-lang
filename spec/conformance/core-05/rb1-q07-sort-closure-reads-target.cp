// expect: E0602 at 2
// rule: 05.7.6
var xs = List(3, 1, 2)
xs.sort(by: x => xs.len - x)
