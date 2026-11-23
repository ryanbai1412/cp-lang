let n: 1..=100 = read()
let a: [0..n]Int = read()
var by_mod: List<List<Int>> = [[] for _ in 0..3]
for x in a:
    by_mod[x % 3].push(x)
print(by_mod[0].len)
