let n: Int = read()
var a: List<Int> = [read() for _ in 0..n]
for x in a:
    a.push(x)
print(sum(a))
