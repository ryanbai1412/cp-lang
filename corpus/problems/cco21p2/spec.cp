// Source: CCO 2021 Day 1 P2 (CCO '21 P2), Weird Numeral System, https://dmoj.ca/problem/cco21p2
// Problem: a base K (2 <= K <= 1e6) and D <= 5001 distinct digits a_i with |a_i| <= M <= 2500.
//   For each of Q <= 5 numbers n (|n| <= 1e18) print, on one line and space-separated, digits
//   most significant first whose base-K value is n (any such; 0 needs at least one digit), or
//   IMPOSSIBLE.
// Solution: The last digit b of x must be congruent to x mod K, and the other digits then
//   spell (x - b) / K. Bucket the digits by residue and search that graph breadth first from x
//   until 0 is reached, recording for each number the digit on the arc that first reached it;
//   the numbers shrink to |y| <= M / (K - 1) + 1, so the search is finite. Walking back from 0
//   reads the digits, most significant first.
let (k, q, d, m): (2..=1e6, 1..=5, 1..=5001, 1..=2500) = read()
domain Residue = 0..k
var ending: [Residue]List<Int> = []  // the digits by residue
for b in read<[d](-m..=m)>(): ending[Residue(b % k)].push(b)

fn spell(x: Int) -> ?List<Int>:
    var via = HashMap<Int, Int>()  // via[y]: the digit b on the arc into y, from y * k + b
    var frontier = [x]
    while 0 not in via:
        if frontier.is_empty: return None
        var next: List<Int> = []
        for a in frontier:
            for b in ending[Residue(a % k)]:
                let rest = (a - b) / k
                if rest not in via:
                    via[rest] = b
                    next.push(rest)
        frontier = next
    var (u, out): (Int, List<Int>) = (0, [])
    while true:
        out.push(via[u])
        u = u * k + via[u]
        if u == x: return out

for x in read<[q](-1e18..=1e18)>():
    if let out = spell(x): print(out)
    else: print("IMPOSSIBLE")
