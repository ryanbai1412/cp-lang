// Source: SDOI 2017 Round 1 (新生舞会, Freshman Ball), https://dmoj.ca/problem/sdoi17r1p4
//   (not public; statement read on Luogu as P3705)
// limits: 1500 ms, 250 MB
// Problem: n <= 100 boys and n girls; pairing boy i with girl j gives a[i][j] and b[i][j]
//   (both 1..1e4). Over perfect matchings, maximise sum a / sum b; print it to 6 decimals.
// Solution: fractional programming by Dinkelbach's method. For a ratio num / den reached by
//   some matching, the best matching under weights den a - num b has weight 0 exactly when
//   num / den is optimal; otherwise its own ratio is strictly larger. So start from num = 0
//   and replace the ratio by the best matching's until its weight is 0: a few Hungarian
//   rounds (`max_weight_matching`, O(n^3) each), exact in integers.
// original: corpus/submissions/5466798.cpp
// Bounds: n: 1..=100; a, b: 1..=1e4

let n: 1..=100 = read()
domain Boy = 0..n
domain Girl = 0..n
let a: [Boy, Girl]1..=1e4 = read()
let b: [Boy, Girl]1..=1e4 = read()

var (num, den) = (0, 1)
loop:
    let best = max_weight_matching<Boy, Girl>((i, j) => den * a[i, j] - num * b[i, j])
    if best.total == 0: break
    (num, den) = (Boy.sum(i => a[i, best.mate[i]]), Boy.sum(i => b[i, best.mate[i]]))
print(fixed(num, den, 6))
