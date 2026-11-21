// SAC '22 CC3 P3 — original: corpus/submissions/4331598.cpp
// Idiomatic CP Lang v2 (locked spec: docs/one-page-cheat-sheet.md)
// Rethought: the original sorts zero-padded digit suffixes as strings; the
// last d digits are just a % 10^d, and a stable sort keeps input order on ties.

let n: 1..=1000 = read()
let a: [0..n]0..=1_000_000_000 = read()

let top = a.max()!
var p = 10
while true:
    // FRICTION 46.11: the spec does not say whether sorted(by:) is stable
    print(a.sorted(by: x => x % p))
    if top < p: break
    p *= 10
