// expect: E0601 at 2
// rule: 05.10.3
var xs = [3, 1]
xs.sort(by: x => xs.len - x)
