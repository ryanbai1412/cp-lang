// expect: E0607 at 3
// rule: 05.10.1
let xs = [3, 4]
let f = () => xs.len
let ys = xs
print(f(), ys.len)
