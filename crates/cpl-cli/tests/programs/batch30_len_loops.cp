fn bump(mut xs: List<Int>): xs.push(0)
fn dot(a: List<Int>, mut b: List<Int>) -> Int:
    var s = 0
    for i in 0..a.len:
        for j in 0..b.len:
            s += a[i] * b[j]
            b[j] += 1
    s
fn grows(n: Int) -> Int:
    var c = [1, 2]
    for k in 0..c.len:
        if c[k] == 1: c.push(5)
    var d = [1, 2, 3]
    for k in 0..d.len:
        d[k] += n
        if k == 1: bump(mut d)
    var e = [4, 5, 6, 7]
    var t = 0
    for k in (0..e.len).rev: t = t * 10 + e[k]
    for k in 1..e.len by 2: t += e[k] + e[k - 1]
    for k in 0..e.len:
        t += e[k]
        if k == 0: e = [1, 1, 1, 1]
    var flags = [true, false, true]
    for k in 0..flags.len if flags[k]: t += 1
    c.len + d.len + d[0] + t
fn shrink() -> Int:
    var xs = [1, 2, 3, 4]
    var s = 0
    for i in 0..xs.len:
        if i == 1: xs.pop()
        print("at", i)
        s += xs[i]
    s
var bb = [3, 4]
print(dot([1, 2], mut bb), bb)
print(grows(3))
print(shrink())
