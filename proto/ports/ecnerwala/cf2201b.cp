// Codeforces 2201 B — Recoloring / Length-2N Construction
//
// A length-`2 * n` array containing each value `1..=n` twice achieves score `k`
// iff `n <= k <= 2 * n - 1`. Set `braid = k - n + 1` (`1 <= braid <= n`):
// - Emit each `v in braid + 1..=n` as an adjacent pair `[v, v]`: these `n - braid`
//   values contribute `1` each.
// - Emit `1..=braid` in a single braided block contributing `2 * braid - 1`:
//   - If `braid == 1`, `[1, 1]` contributes `1`.
//   - If `braid >= 2`, start with `[1, 2]`, append `[i + 2, i]` for
//     `i in 1..braid - 1` (each step introduces `i + 2` before closing `i`),
//     and finish with `[braid - 1, braid]`. Every value `v in 1..=braid`
//     appears twice with strictly overlapping spans and no equal neighbours,
//     giving `(n - braid) + (2 * braid - 1) = k` in total.

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
