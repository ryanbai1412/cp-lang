// expect: E0607
// rule: 05.10.3
var xs = List(1, 2)
let f = () => xs.len
let ys = xs // <- here
print(f())
