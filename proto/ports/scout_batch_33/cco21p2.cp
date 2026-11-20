// CCO 2021 P2, Weird Numeral System: a base k and a set of d allowed
// digits, which may be negative. For each of q numbers print digits, most
// significant first, whose base-k value is the number, or IMPOSSIBLE.
//
// The last digit b of x must agree with x modulo k, and the rest of the
// digits then spell (x - b) / k. Search that graph breadth first from x to
// 0, remembering for each number reached the digit that led to it; the
// digits are bounded, so the numbers shrink towards a small range and 100
// rounds are plenty. Reading the digits back from 0 spells the answer.
// input: k q d m, the d digits, then the q numbers (m, the digit bound, is unused)
// original: corpus/submissions/4543410.cpp
let (k, q, d, m): (Int, Int, Int, Int) = read()
domain Digit = 0..d
domain Residue = 0..k
let digits: [Digit]Int = read()
var ending: [Residue]List<Int> = [Residue] _ => []      // the digits by residue
for i in Digit: ending[Residue(digits[i] % k)].push(digits[i])

fn spell(x: Int) -> ?List<Int>:
    // WISH C4: `HashMap<Int, Int>` is on the cheat sheet; the interpreter only has `Map`.
    var via = Map<Int, Int>()           // via[y]: a digit b with y * k + b reached
    var frontier = [x]
    for _ in 0..100:
        var next: List<Int> = []
        for a in frontier:
            for b in ending[Residue(a % k)]:
                let rest = (a - b) / k
                if rest not in via:
                    via[rest] = b
                    next.push(rest)
        if 0 in via:
            var (u, out): (Int, List<Int>) = (0, [])
            while true:
                let b = via[u]!
                out.push(b)
                u = u * k + b
                if u == x: return out
        frontier = next
    none

for _ in 0..q:
    let x: Int = read()
    if let out = spell(x):
        for b in out: print(b)
    else: print("IMPOSSIBLE")
