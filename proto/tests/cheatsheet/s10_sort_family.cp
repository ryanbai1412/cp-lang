let n: 1..=10 = read()
domain I = 0..n
var a: [I]Int = read()
let b = sorted(a, by: x => -x)
sort(mut a); reverse(mut a)
print(b, a)
