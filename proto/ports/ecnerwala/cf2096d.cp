// Codeforces 2096 D: every vertical pair shares x and every diagonal pair
// shares x + y, so XORing all x and all x + y recovers the unmatched bulb.
@!multitest
let n: Int = read()
let pts: [0..n](x: Int, y: Int) = read()
let x = fold(^, p.x for p in pts)
let diag = fold(^, p.x + p.y for p in pts)
print(x, diag - x)
