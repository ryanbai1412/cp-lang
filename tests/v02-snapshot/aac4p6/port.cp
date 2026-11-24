// Source: An Animal Contest 4 P6 (Cozy Cottages), https://dmoj.ca/problem/aac4p6
// Problem: N <= 1e6 cottages on a circle with a_i <= 1e9 elves. Each of K <= 1e9 hours every
//   non-empty cottage sends exactly one elf to the next cottage, all at once. Print the final
//   counts.
// Solution: after min(N, K) hours either no cottage is empty, and then nothing changes any
//   more, or the elves stand at most one per cottage and the pattern turns one cottage per
//   hour: simulate m = min(N, K) hours and, if K > N with an empty cottage, rotate by K mod N.
//   The m hours take one O(N + m) sweep: a deque holds the hours at which no elf enters the
//   current cottage (seeded by running the elves of cottages N-1, N-2, ... in reverse); each
//   such hole costs the cottage an elf, and its own a_i elves fill the next a_i holes of the
//   stream it passes on.
// original: corpus/submissions/4227368.cpp

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
            a[i] max= 1
        dl -= 1
        buf[dl] = -i - 1
        var drop = pi
        drop min= dr - dl
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
