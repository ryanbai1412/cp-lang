@!multitest
let n: Int = read()
var a: List<Int> = [read::<Int>() for _ in 0..n]
let tot = sum(x for x in a)
if (tot & 1) == 1:
    print(-1)
    return
var cur = 0
var mid = -1
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
var b = List.filled(n, 0)
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
