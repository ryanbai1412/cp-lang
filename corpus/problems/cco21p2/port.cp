// Source: CCO 2021 Day 1 P2 (CCO '21 P2), Weird Numeral System, https://dmoj.ca/problem/cco21p2
// Problem: a base K (2 <= K <= 1e6) and D <= 5001 distinct digits a_i with |a_i| <= M <= 2500.
//   For each of Q <= 5 numbers n (|n| <= 1e18) print, on one line and space-separated, digits
//   most significant first whose base-K value is n (any such; 0 needs at least one digit), or
//   IMPOSSIBLE.
// Solution: the last digit b of x must be congruent to x mod K, and the other digits then
//   spell (x - b) / K. Bucket the digits by residue and search that graph breadth first from x
//   until 0 is reached by at least one digit, recording for each number the digit that led to
//   it (a map, the numbers are unbounded). The numbers shrink to |y| <= M / (K - 1) + 1 within
//   about 60 steps, so the search is finite. Walking back from 0 reads the digits, most
//   significant first.
// original: corpus/submissions/4543410.cpp
let (k, q, d, m): (2..=1e6, 1..=5, 1..=5001, 1..=2500) = read()
domain Residue = 0..k
let digits: [d](-m..=m) = read()
var ending: [Residue]List<Int> = []
for b in digits: ending[Residue(b % k)].push(b)

fn spell(x: Int) -> ?List<Int>:
    var via = HashMap<Int, Int>()  // via[y] = b: y * k + b was reached
    var frontier = [x]
    while not frontier.is_empty and 0 not in via:
        var next: List<Int> = []
        for y in frontier:
            for b in ending[Residue(y % k)]:
                let rest = (y - b) / k
                if rest not in via:
                    via[rest] = b
                    next.push(rest)
        frontier = next
    if 0 not in via: return None
    var out: List<Int> = []
    var y = 0
    while true:
        let b = via[y]
        out.push(b)
        y = y * k + b
        if y == x: return out
    None

for _ in 0..q:
    let x: -1e18..=1e18 = read()
    print(spell(x), none: "IMPOSSIBLE")
