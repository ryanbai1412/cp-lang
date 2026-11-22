// Codeforces 2180 E — No Effect XOR
//
// Count positive integers `x > 0` such that `v ^ x in [l, r)` for every
// `v in [l, r)`, where `r = r0 + 1` is the half-open upper bound (`1 <= l <= r0`).
//
// - Low bits: let `b = min(ctz(l), ctz(r))`. Since `2^b` divides both `l` and
//   `r`, `[l, r)` is a union of aligned `2^b`-blocks, and XORing by any value
//   using only bits `< b` permutes each block internally. Thus all `2^b` such
//   shifts preserve `[l, r)` (and no higher bit below the split bit can be set
//   without moving a boundary block outside `[l, r)`).
// - Split bit: let `half = 1 << (bit_width(l ^ r) - 1)` be the highest bit
//   where `l` and `r` differ, dividing `[l, r)` at its aligned midpoint `M`.
//   Flipping that bit swaps `[l, M)` with `[M, M + (M - l))`, which lands
//   exactly on `[l, r)` iff `M - l == r - M`, i.e.
//   `(l & (half - 1)) + (r & (half - 1)) == half`. When symmetric, combining
//   that flip with any low `2^b` shift doubles the count (excluding `x = 0`).

@!multitest
let (l, r0): (1..=2 ** 60, 1..=2 ** 60) = read()
let r = r0 + 1
let half = 1 << (bit_width(l ^ r) - 1)
let sym = (l & (half - 1)) + (r & (half - 1)) == half
print(((1 + Int(sym)) << min(ctz(l), ctz(r))) - 1)
