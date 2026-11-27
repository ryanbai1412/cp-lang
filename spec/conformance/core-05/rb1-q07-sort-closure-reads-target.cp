// expect: E0602
// rule: 05.7.6
var xs = [3, 1, 2]
xs.sort(by: x => xs.len - x) // <- here
