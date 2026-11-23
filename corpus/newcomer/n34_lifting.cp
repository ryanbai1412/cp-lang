let (n, q): (1..=200000, 1..=200000) = read()
let p: [0..n]0..n = read()
var up: [18, n]Int = 0
for v in 0..n:
    up[0, v] = p[v]
for k in 1..18:
    for v in 0..n:
        up[k, v] = up[k - 1, up[k - 1, v]]  // expect: hand_rolled_binary_lifting
for _ in 0..q:
    let (v0, d): (0..n, 0..=200000) = read()
    var v = v0
    for k in 0..18:
        if (d >> k) & 1 == 1:  // expect: missing manual-bit-test
            v = up[k, v]
    print(v)
