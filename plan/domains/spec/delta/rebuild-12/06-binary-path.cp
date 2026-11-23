// 1937B Binary Path
// in: t; per test n, then two binary strings (rows). out: smallest path string, then its path count
@!multitest
let n: Int = read()
let a: Str = read()
let b: Str = read()
let down = (0..n - 1).find(j => a[j + 1] == '1' and b[j] == '0') ?? n - 1
var j = down
while j > 0 and a[j] == b[j - 1]:
    j -= 1
print(a[0..=down] + b[down..])
print(down - j + 1)
