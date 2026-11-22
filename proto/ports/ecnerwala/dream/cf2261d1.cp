// Codeforces 2261 D1 — Unrestricted Half-Shift Pairing
//
// With `n == 1`, no distinct pair of positions exists so the answer is `0`.
// For `n >= 2`, covering all `n` positions requires at least `half = ceil(n / 2)`
// pairs. Pairing each `p in Pos(1)..=Pos(half)` with `p.offset(half - 1)!`
// (`1 + (half - 1) ..= half + (half - 1) <= n`) covers every position in
// `1..=n` (sharing only the middle position when `n` is odd), achieving the
// `ceil(n / 2)` lower bound.

@!multitest
let (n, _m): (1..=200_000, 0..=0) = read()
if n == 1:
    print(0)
    return
domain Pos = 1..=n
let half = ceil_div(n, 2)
print(half)
for p in Pos(1)..=Pos(half): print(p, p.offset(half - 1)!)
