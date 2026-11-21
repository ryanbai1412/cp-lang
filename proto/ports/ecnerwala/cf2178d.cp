@!multitest
let (n, m): (Int, Int) = read()
let vals: [0..n]Int = read()
let ord = sorted(0..n, by: i => vals[i])

if 2 * m > n:
    print(-1)
elif m >= 1:
    print(n - m)
    for i in 0..n - 2 * m: print(ord[i] + 1, ord[i + 1] + 1)
    for i in 0..m: print(ord[n - 1 - i] + 1, ord[n - 1 - m - i] + 1)
else:
    var s = n - 2
    var r = vals[ord[n - 1]]
    while s >= 0 and r > 0:
        r -= vals[ord[s]]
        s -= 1
    if r > 0:
        print(-1)
        return
    print(n - 1)
    s += 1
    for i in 0..s: print(ord[i] + 1, ord[i + 1] + 1)
    for i in (s..n - 1).rev: print(ord[i] + 1, ord[n - 1] + 1)
