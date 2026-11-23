let x0: 2..=1000000000000 = read()
var x = x0
var p = 2
var fs: List<Int> = []
while p * p <= x:  // expect: missing factor_by_hand
    while x % p == 0:
        fs.push(p)
        x /= p
    p += 1
if x > 1:
    fs.push(x)
print(fs)
