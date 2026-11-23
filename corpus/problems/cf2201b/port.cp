// Source: Codeforces 2201B, Recollect Numbers, https://codeforces.com/contest/2201/problem/B
// Problem: multitest, n <= 300000, k <= 1e6. Arrange cards 1, 1, ..., n, n in a row so that the
//   statement's greedy flipping strategy takes exactly k turns; print YES and the row, or NO.
// Solution: possible iff n <= k <= 2n - 1. With braid = k - n + 1, values above braid go first
//   as adjacent pairs [v, v], one turn each; 1..=braid form a braided block [1, 2], [i + 2, i],
//   ..., [braid - 1, braid] of overlapping spans that takes 2 braid - 1 turns. O(n).

@!multitest
let (n, k): (1..=300_000, 1..=600_000) = read()
if not (n <= k <= 2 * n - 1):
    print("NO")
    return

print("YES")
let braid = k - n + 1
var out: List<Int> = []
for v in (braid + 1..=n).rev: out.extend([v, v])
if braid == 1:
    out.extend([1, 1])
else:
    out.extend([1, 2])
    for i in 1..braid - 1: out.extend([i + 2, i])
    out.extend([braid - 1, braid])
print(out)
