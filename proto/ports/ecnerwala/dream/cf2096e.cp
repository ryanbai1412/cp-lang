// Codeforces 2096 E — Wonderful Teddy Bears
//
// Sort `'B'`s before `'P'`s using length-3 block swaps (which move a `'B'`
// left by 2, removing 2 inversions while preserving every character's index
// parity) and adjacent swaps (removing 1 inversion and flipping parity).
//
// Scanning `s` in reverse (`i = 0..n` from the right), `cnt_b` counts `'B'`s
// to the right of the current position, so each `'P'` adds `cnt_b` inversions
// to `inv`. In the sorted target, the `n - cnt_b` `'P'`s occupy reversed
// indices `0..(n - cnt_b)`, of which `(n - cnt_b) / 2` are odd. Hence
// `parity_fix = abs(cnt_p_odd - (n - cnt_b) / 2)` single-step swaps are forced,
// and the remaining `inv - parity_fix` inversions pair into length-3 swaps,
// giving `parity_fix + (inv - parity_fix) / 2 = (inv + parity_fix) / 2` moves.

@!multitest
let n: 1..=200_000 = read()
let s: [0..n]Char = read()
var cnt_b = 0
var cnt_p_odd = 0
var inv = 0
for (i, c) in s.rev.items:
    if c == 'P':
        inv += cnt_b
        cnt_p_odd += i % 2
    else:
        cnt_b += 1
let parity_fix = abs(cnt_p_odd - (n - cnt_b) / 2)
print((inv + parity_fix) / 2)
