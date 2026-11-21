@!multitest
let n: Int = read()
var a: [0..n]Int = read()
let tot = sum(a)
if tot % 2 == 1:
    print(-1)
    return

var cur = a[0]
var mid = 0
while cur < tot / 2:
    mid += 1
    cur += a[mid]

if cur == tot / 2:
    print(1)
    print(a)
    return
if a[mid] * 2 > tot:
    print(-1)
    return

var b: [0..n]Int = 0
var rem = cur - tot / 2
for i in 0..mid:
    let v = min(a[i], rem)
    b[i] += v
    b[mid] += v
    a[i] -= v
    a[mid] -= v
    rem -= v

print(2)
print(a)
print(b)
