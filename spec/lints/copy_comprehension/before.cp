let n: 1..=100 = read()
let a: [0..n]Int = read()
var b: List<Int> = [x for x in a]
b.push(0)
let c: List<Int> = [x for x in 0..n by 4]
print(b, c)
