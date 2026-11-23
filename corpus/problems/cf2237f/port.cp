// Codeforces 2237 F — Length-M Block Alignment DP
//
// Partition `0..n` so every kept index `i` sits at offset `a[i] - 1` inside a
// length-`m` block `[block_start .. block_start + m)` where
// `block_start = i - (a[i] - 1) in 0..=n - m`, and the array contains at least
// one full length-`m` block. Maximizing kept elements minimizes edits `n - kept`.
//
// Three DP tables:
// - `ending_at[i]` (`?Int`): max kept elements ending with `i` kept.
// - `at_boundary[i]` (`Int`, seeded `0`): prefix max of `ending_at[j]` over
//   `j < i` (0 when no element is kept yet) — any kept prefix ending before
//   `block_start` can start a new block at `block_start`.
// - `after_block[i]` (`?Int`, seeded `after_block[m] = 0`): max kept elements
//   where some full length-`m` block has already finished at or before `i`
//   (seeding `after_block[m] = 0` allows skipping an initial length-`m` block
//   with 0 kept elements).
// - `last_with_start[st]`: last index `p < i` sharing `block_start == st`,
//   allowing `i` to continue the same block (`ending_at[p] + 1`).
//
// Final answer: either a full length-`m` block finishes at or before `n`
// (`after_block[n]`), or a final untouched length-`m` block sits in `[n - m..n)`
// after a kept prefix ending before `n - m` (`at_boundary[n - m]`).

@!multitest
let (n, m): (1..=200_000, 1..=200_000) = read()
let a: [0..n]1..=m = read()
var at_boundary: [0..=n]Int = 0
var after_block: [0..=n]?Int = None
var ending_at: [0..=n]?Int = None
var last_with_start: [0..=n - m]?Int = None

after_block[m] = 0
for i in 0..n:
    at_boundary[i + 1] max= at_boundary[i]
    after_block[i + 1] max= after_block[i]
    let block_start = i - (a[i] - 1)
    if 0 <= block_start <= n - m:
        ending_at[i] max= at_boundary[block_start] + 1
        if let v = after_block[i]: ending_at[i] max= v + 1
        if let p = last_with_start[block_start] and let v = ending_at[p]:
            ending_at[i] max= v + 1
        last_with_start[block_start] = i
        after_block[block_start + m] max= ending_at[i]
        if let v = ending_at[i]: at_boundary[i + 1] max= v
print(n - max(at_boundary[n - m], after_block[n] ?? 0))
