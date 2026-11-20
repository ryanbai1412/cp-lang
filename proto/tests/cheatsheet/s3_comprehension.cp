// cp: 0.1 bea787c+dirty
let n: 1..=10 = read()
domain V = 1..=n
let a: [V]Int = read()
let v = V(2)
let b: [V]Int = [V] v => a[v] * 2
print(b)
