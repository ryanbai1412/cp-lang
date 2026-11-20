@!multitest
let n: Int = read()
let a: [0..n]Int = read()
var seen = Set<Int>()
for x in a: seen.insert(x)
print(2 * seen.len - 1)
