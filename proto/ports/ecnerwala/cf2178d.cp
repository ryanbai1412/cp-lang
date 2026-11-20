@!multitest
let (n, m): (Int, Int) = read()
let vals: [0..n]Int = read()
var a = [(vals[i], i) for i in 0..n]
sort(mut a)
if 2 * m > n:
    print(-1)
elif m >= 1:
    print(n - m)
    for i in 0..(n - 2 * m):
        print(a[i].1 + 1, a[i + 1].1 + 1)
    for i in 0..m:
        print(a[n - 1 - i].1 + 1, a[n - 1 - m - i].1 + 1)
else:
    var s = n - 2
    var r = a[n - 1].0
    while s >= 0 and r > 0:
        r -= a[s].0
        s -= 1
    if r <= 0:
        print(n - 1)
        s += 1
        for i in 0..s:
            print(a[i].1 + 1, a[i + 1].1 + 1)
        for i in (s..(n - 1)).rev:
            print(a[i].1 + 1, a[n - 1].1 + 1)
    else:
        print(-1)
