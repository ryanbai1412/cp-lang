// expect: E0602 at 3
// rule: 05.8.4 07.5.9
var xs = [1, 2]
let x = mut xs[0]
xs.push(3)
x += 1
