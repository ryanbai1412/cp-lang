fn kernel(n: Int, q: Int, a: List<Int>, qs: List<(Int, Int, Int)>) -> Int:
    var p = [0 for _ in 0..=n]
    for i in 0..n: p[i + 1] = p[i] + a[i]
    var size = 1
    while size < n: size *= 2
    var t = [0 for _ in 0..(2 * size)]
    for i in 0..n: t[size + i] = a[i]
    var k = size - 1
    while k >= 1:
        t[k] = t[2 * k] + t[2 * k + 1]
        k -= 1
    var acc = 0
    for (kind, x, y) in qs:
        if kind == 1:
            var idx = x + size
            t[idx] = y
            idx >>= 1
            while idx >= 1:
                t[idx] = t[2 * idx] + t[2 * idx + 1]
                idx >>= 1
        else:
            var l = x + size
            var r = y + 1 + size
            var s = 0
            while l < r:
                if (l & 1) == 1:
                    s += t[l]
                    l += 1
                if (r & 1) == 1:
                    r -= 1
                    s += t[r]
                l >>= 1
                r >>= 1
            acc = (acc + s + (p[y + 1] - p[x])) ^ (acc >> 4)
    acc

let n: Int = read()
let q: Int = read()
let a: [0..n]Int = read()
var qs: List<(Int, Int, Int)> = []
for _ in 0..q:
    let (t, x, y): (Int, Int, Int) = read()
    qs.push((t, x, y))
print(kernel(n, q, a, qs))
