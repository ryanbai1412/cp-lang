// AAC 4 P6 (today/ runnable subset) — original: corpus/submissions/4227368.cpp

let n: 1..=1_000_000 = read()
let k: 0..=1_000_000_000 = read()
var a: [0..n]Int = read()

if k == 0:
    print(a)
else:
    let m = min(n, k)
    let cap = 2 * n + m + 5
    var buf: [0..cap]Int = 0
    var dl = n + 2
    var dr = n + 2

    var csum = 0
    for j in 0..m:
        csum += a[n - 1 - j]
        if csum > 0:
            csum -= 1
        else:
            buf[dr] = j
            dr += 1

    for i in 0..n:
        let pi = a[i]
        while dr > dl and buf[dr - 1] >= m - i:
            dr -= 1
        let sz = dr - dl
        a[i] = max(0, a[i] - sz)
        if dr == dl or buf[dr - 1] != m - i - 1:
            if a[i] < 1:
                a[i] = 1
        dl -= 1
        buf[dl] = -i - 1
        var drop = pi
        if drop > dr - dl:
            drop = dr - dl
        dl += drop

    var has_zero = false
    for i in 0..n:
        if a[i] == 0:
            has_zero = true

    if k > n and has_zero:
        let shift = (n - (k % n)) % n
        var rot: [0..n]Int = 0
        for i in 0..n:
            rot[i] = a[(i + shift) % n]
        print(rot)
    else:
        print(a)
