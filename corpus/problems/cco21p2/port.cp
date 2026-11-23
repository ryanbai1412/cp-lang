// Source: CCO 2021 Day 1 P2 (CCO '21 P2), Weird Numeral System, https://dmoj.ca/problem/cco21p2
// Problem: a base K (2 <= K <= 1e6) and D <= 5001 distinct digits a_i with |a_i| <= M <= 2500.
//   For each of Q <= 5 numbers n (|n| <= 1e18) print digits, most significant first, whose
//   base-K value is n (any such; 0 needs at least one digit), or IMPOSSIBLE.
// Solution: The last digit b of x must be congruent to x mod K, and the other digits then
//   spell (x - b) / K. Bucket the digits by residue and search that graph breadth first from x
//   to 0, recording for each number reached the digit that led to it (a map); the numbers
//   shrink to |y| <= M / (K - 1) + 1 within about 60 steps and 100 rounds suffice. Walking back
//   from 0 reads the digits, most significant first.
// input: k q d m, the d digits, then the q numbers (m, the digit bound, is unused)
// original: corpus/submissions/4543410.cpp
let (k, q, d, m): (Int, Int, Int, Int) = read()
domain Digit = 0..d
domain Residue = 0..k
let digits: [Digit]Int = read()
var ending: [Residue]List<Int> = [Residue] _ => []  // the digits by residue
for i in Digit: ending[Residue(digits[i] % k)].push(digits[i])

fn spell(x: Int) -> ?List<Int>:
    // WISH C4: `HashMap<Int, Int>` is on the cheat sheet; the interpreter only has `Map`.
    var via = Map<Int, Int>()  // via[y]: a digit b with y * k + b reached
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
                let b = via[u]
                out.push(b)
                u = u * k + b
                if u == x: return out
        frontier = next
    None

for _ in 0..q:
    let x: Int = read()
    if let out = spell(x):
        for b in out: print(b)
    else: print("IMPOSSIBLE")
