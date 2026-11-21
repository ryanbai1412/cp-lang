let n: Int = read()
let a: [0..n]Int = read()
var s = 0
for i in 0..n:
    s += a[i - 1]
print(s)
