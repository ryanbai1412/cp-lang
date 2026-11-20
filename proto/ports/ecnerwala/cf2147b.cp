@!multitest
let n: Int = read()
var out: List<Int> = [n - i for i in 1..n]
out.push(n)
for i in 1..n: out.push(i)
out.push(n)
print(out)
