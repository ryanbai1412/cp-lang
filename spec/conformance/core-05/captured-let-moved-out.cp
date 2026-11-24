// expect: E0607
// rule: 05.10.1
let xs = [3, 4]
let f = () => xs.len
let ys = xs // <- here
print(f(), ys.len)
