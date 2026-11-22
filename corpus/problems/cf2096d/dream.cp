// Codeforces 2096 D — Wonderful Lightbulbs
//
// Each operation toggles two pairs of bulbs that share the same `x` coordinate
// and two pairs that share the same diagonal `x + y`. Hence the parity of every
// column `x` and every diagonal `x + y` is invariant: XORing all lit `x` gives
// the unique treasure's `x`, XORing all lit `x + y` gives its `diag = x + y`,
// and `y = diag - x` recovers its `y` coordinate.

@!multitest
let n: 1..=200_000 = read()
let pts: [0..n](x: -10 ** 9..=10 ** 9, y: -10 ** 9..=10 ** 9) = read()
let x = fold(^, p.x for p in pts)
let diag = fold(^, p.x + p.y for p in pts)
let y = diag - x
print(x, y)
