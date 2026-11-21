// ETSR '23 Round 2 P3 — Trailing-Zero Divisibility Triple (CP Lang v2 spec)
// original: corpus/submissions/5494453.cpp
// spec: docs/one-page-cheat-sheet.md (3723dca)
//
// Sort the triple (a, b, c) and check whether both consecutive gaps are
// divisible by 2^(ctz(min) + 1).

@multitest
var xs: [0..3]Int = read()
xs.sort()
let step = 1 << (ctz(xs[0]) + 1)
let ok = (xs[1] - xs[0]) % step == 0 and (xs[2] - xs[1]) % step == 0
print(if ok: "YES" else: "NO")
