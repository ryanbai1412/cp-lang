// expect: E0602 at 2
var xs = List(3, 1, 2)
xs.sort(by: x => xs.len - x)
