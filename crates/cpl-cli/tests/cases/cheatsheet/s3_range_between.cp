// cp: 0.1 bea787c+dirty
let n: 1..=10 = read()
domain V = 1..=n
let a: [V]Int = read()
let v = V(2)
print(sum(a[i] for i in v.r..V(4).l))
