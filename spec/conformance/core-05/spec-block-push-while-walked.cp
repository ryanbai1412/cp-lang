// expect: E0602 at 2
// rule: 05.9.4
var xs = List(1, 2)
for x in xs: xs.push(x)          // error: xs is read-lent to the loop
for i in xs.keys: xs.push(xs[i]) // fine: xs.keys is a plain range, fixed before the loop
