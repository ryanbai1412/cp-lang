// cp: 0.1 bea787c+dirty
let n: 1..=10 = read()
domain I = 0..n
var a: [I]Int = read()
print(sum(x for x in a), any(x > 2 for x in a), all(x > 2 for x in a), count(x > 2 for x in a), min(x for x in a), max(x for x in a))
