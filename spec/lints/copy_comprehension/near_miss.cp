let n: 1..=100 = read()
let a: [0..n]Int = read()
var b: List<Int> = [x * 2 for x in a]
b.push(0)
print(b)
let every_other = [i for i in 0..n by 2]
print(every_other)
