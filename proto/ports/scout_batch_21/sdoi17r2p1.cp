// SDOI 2017 Round 2 Dragons and Dungeons: roll n dice whose faces are
// 0..m - 1, each face equally likely. For ten ranges lo..=hi, print the
// probability that the total lands in the range.
//
// One die at a time: the total after another die is the average of m
// neighbouring totals before it, a window over the prefix sums of the
// previous row. The original multiplies polynomials by repeated squaring
// and skips products of two tiny terms (below 1e-5) to stay fast; the
// window needs no threshold and drops nothing.
// original: corpus/submissions/5481972.cpp
use real.float.{fixed}

let tests: Int = read()
for _ in 0..tests:
    let m: Int = read()
    let n: Int = read()
    domain Total = 0..=n * (m - 1)
    var chance: [Total]F64 = 0.0
    chance[Total(0)] = 1.0
    for die in 1..=n:
        var next: [Total]F64 = 0.0
        var window = 0.0                    // chance[t - m + 1] + ... + chance[t]
        for t in Total.first..=Total(die * (m - 1)):
            window += chance[t]
            if let out = t.offset(-m): window -= chance[out]
            next[t] = window / m
        chance = next
    var below: List<F64> = [0.0]          // below[k]: the total is less than k
    for t in Total: below.push(below.last + chance[t])
    for _ in 0..10:
        let lo: Int = read()
        let hi: Int = read()
        print(fixed(below[hi + 1] - below[lo], 6))
