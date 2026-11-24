// expect: E0602
// rule: 05.8.4
var xs = List(3, 1, 2)
xs.sort(by: x => xs.len - x) // <- here
