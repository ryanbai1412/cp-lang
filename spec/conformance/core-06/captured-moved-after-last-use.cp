// expect: ok, output "2 2"
// rule: 06.2.2
let xs = [1, 2]
let f = () => xs.len
let n = f()
let ys = xs
print(n, ys.len)
