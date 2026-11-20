// cp: 0.1 38a7ef4+dirty
let n: 1..=9 = read()
domain I = 1..=n
let a: [I]0..=100 = read()
let b = [I] i => a[i.mirror]
print(I.at(1).idx, I.at(n).idx, I.nth(0).idx, b)
