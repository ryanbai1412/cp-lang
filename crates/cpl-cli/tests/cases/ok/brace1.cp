// cp: 0.1 77bef7c+dirty
let n: 1..=10 = read()
domain I = 1..=n
let a: [I]-100..=100 = read()
let lim = { let s = sum(a[i] for i in I); s * 2 + 1 }
let order = [I] i => { let x = a[i]; x * x - i.idx }
print(lim, order)
