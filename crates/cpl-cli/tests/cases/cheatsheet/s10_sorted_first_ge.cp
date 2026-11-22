// cp: 0.1 bea787c+dirty
let n: 1..=10 = read()
domain I = 0..n
var a: [I]Int = read()
sort(mut a)
print(a.first_ge(2), a.last_le(2))
