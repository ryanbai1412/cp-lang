let n: 1..=10 = read()
domain V = 1..=n
let a: [V]Int = read()
let v = V(2)
let r = V(2)..V(2)
print(r.first ?? V(1), r.last ?? V(1))
