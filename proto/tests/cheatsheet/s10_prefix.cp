// cp: 0.1 bea787c+dirty
let n: 1..=10 = read()
domain I = 0..n
var a: [I]Int = read()
let pre = prefix(a)
print(pre.sum(I(1)..=I(3)), pre.sum(..I(2)))
