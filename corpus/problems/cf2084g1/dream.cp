// Codeforces 2084 G1: Wish Upon a Satellite (Easy Version).
//
// Beauty = sum over k in 0..n of the number of segments whose f exceeds k.
// At threshold k call the k smallest values small and the rest large, and
// let d(X) = (#X on even positions) - (#X on odd positions). Exactly
// ((n + 1)(n - k) + d(small) * d(large)) / 2 segments have f > k (brute
// force agrees for every permutation with n <= 6). Summed over k the first
// term is n(n + 1)^2 / 2, so b only matters through the products, and they
// only see which values sit on even positions.
//
// So place the values in increasing order. With c of the k smallest on the
// n / 2 even positions, d(small) = 2c - k and d(large) = 2(n/2 - c) - (n - k).
// best[c] is the largest sum of products so far; a value fixed by `a` keeps
// the side of its position. O(n^2).
enum Side = Odd | Even

@!multitest
let n: 1..=5000 = read()
domain P = 1..=n
let a: [P]?(1..=n) = read(None: 0)
let half = n / 2
let at = a.inverse()
fn side(p: P) -> Side = if p.label % 2 == 0: Even else: Odd

var best: [0..=half]?Int = None
best[0] = 0
for k in 0..n:
    let forced = at[k + 1].map(side)
    var next: [0..=half]?Int = None
    for c in 0..=min(k, half):
        let product = (2 * c - k) * (2 * (half - c) - (n - k))
        for (s, to) in [(Odd, c), (Even, c + 1)] if (forced ?? s) == s and to <= half:
            next[to] max= best[c] + product
    best = next
print((best[half]! + n * (n + 1) ** 2 / 2) / 2)
