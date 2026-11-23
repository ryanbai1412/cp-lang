// Source: SDOI 2017 Round 2, Dragons and Dungeons (not on DMOJ; Luogu P3779)
// Problem: T <= 10 tests: Y <= 2e5 dice with faces 0..X-1 (2 <= X <= 20) uniform; for 10
//   queries A <= B print P(A <= total <= B), absolute error 0.013579. At most 2 tests have
//   Y > 800.
// Solution: one die at a time: the next distribution is the average of X neighbouring totals,
//   a window over prefix sums of the previous row. Exact and drops nothing (the original
//   squares polynomials and skips products below 1e-5), but O(X Y^2) per test.
// original: corpus/submissions/5481972.cpp

let tests: Int = read()
for _ in 0..tests:
    let m: Int = read()
    let n: Int = read()
    domain Total = 0..=n * (m - 1)
    var chance: [Total]F64 = 0.0
    chance[Total(0)] = 1.0
    for die in 1..=n:
        var next: [Total]F64 = 0.0
        var window = 0.0  // chance[t - m + 1] + ... + chance[t]
        for t in Total.first!..=Total(die * (m - 1)):
            window += chance[t]
            if let out = t.offset(-m): window -= chance[out]
            next[t] = window / m
        chance = next
    var below: List<F64> = [0.0]  // below[k]: the total is less than k
    for t in Total: below.push(below.last + chance[t])
    for _ in 0..10:
        let lo: Int = read()
        let hi: Int = read()
        print(below[hi + 1] - below[lo], precision: 6)
