// expect: E0607
// rule: 05.10.2 06.2.2
let xs = [1, 2]
let f = () => xs.len
let ys = xs // <- here
print(f() + ys.len)
