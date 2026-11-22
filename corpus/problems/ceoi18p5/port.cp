// CEOI 2018 Toys: write n as a product of any number of factors >= 2 (n = 1
// is the empty product); a factorisation scores the sum of (factor - 1).
// Print how many different scores exist, then the scores in increasing order.
// Enumerate factorisations with non-decreasing factors over the divisor list.
// original: corpus/submissions/5323876.cpp
let n: 1..=1e9 = read()

// std.math.divisors, inlined: the prototype compiler cannot import it yet.
fn divisors(x: Int) -> List<Int>:
    var small: List<Int> = []
    var large: List<Int> = []
    var d = 1
    while d * d <= x:
        if x % d == 0:
            small.push(d)
            if d * d != x: large.push(x / d)
        d += 1
    for q in large.rev: small.push(q)
    small

fn scores(n: Int) -> List<Int>:
    let divs = divisors(n)
    var found = Set<Int>()
    // `rest` is still to be split into factors >= divs[from]; `score` so far.
    fn split(rest: Int, from: Int, score: Int):
        found.insert(score + rest - 1)  // keep `rest` as the last factor
        for k in from..divs.len:
            let d = divs[k]
            if d * d > rest: break
            if rest % d == 0: split(rest / d, k, score + d - 1)
    if n == 1: found.insert(0)
    else: split(n, 1, 0)
    [s for s in found]

let all = scores(n)
print(all.len)
print(all)
