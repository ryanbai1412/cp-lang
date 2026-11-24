// expect: E0602
// rule: 05.8.4 07.5.9
var xs = [1, 2]
let x = mut xs[0]
xs.push(3) // <- here
x += 1
