// Source: Codeforces 2165D (Path Split), https://codeforces.com/contest/2165/problem/D
// Problem: split a_1..a_n (1 <= a_i <= 2n) into the fewest subsequences whose consecutive
//   elements differ by exactly 1; print that number. n <= 1e6 (sum), multitest.
// Solution: the answer is n minus the most links i -> j (i < j, |a_i - a_j| = 1) with every
//   index in at most one incoming and one outgoing link. Links join even and odd values,
//   so the two degree limits split into two independent matchings between values v - 1 and v
//   (by which side's out-degree is used). Reflecting the positions of one parity turns both
//   directions into "earlier carried position < pos"; sweep v = 0..=2n and greedily match
//   each position of v to any carried unmatched position of v - 1 before it. O(n) per test.
// Bounds: n: 1..=1_000_000; vals: 1..=2 * n

enum Flip:
    Even
    Odd

@!multitest
let n: 1..=1_000_000 = read()
let vals: [0..n]1..=2 * n = read()
var by_val: [0..=2 * n]List<Int> = []
for (i, v) in vals.items: by_val[v].push(i)

fn reflect(xs: List<Int>) -> List<Int> = [n - 1 - x for x in xs.rev]

var ans = n
for flip in [Flip.Even, Flip.Odd]:
    var unmatched_prev: List<Int> = []
    for v in 0..=2 * n:
        let is_flipped = match flip:
            Flip.Even => v % 2 == 0
            Flip.Odd => v % 2 == 1
        let positions = if is_flipped: reflect(by_val[v]) else: by_val[v]
        let carried = reflect(unmatched_prev)
        var (ptr, available) = (0, 0)
        var unmatched: List<Int> = []
        for pos in positions:
            while ptr < carried.len and carried[ptr] < pos:
                ptr += 1
                available += 1
            if available > 0:
                available -= 1
                ans -= 1
            else:
                unmatched.push(pos)
        unmatched_prev = unmatched
print(ans)
