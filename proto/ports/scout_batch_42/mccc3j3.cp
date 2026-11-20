// Mock CCC '22 Contest 1 J3 — Maximum Character-Frequency Dot Product (CP Lang v2 spec)
// original: corpus/submissions/4223424.cpp
// spec: docs/one-page-cheat-sheet.md (3723dca)
//
// Maximize sum_c (count(a, c) * count(b, c)) after replacing at most one character of b.
// Rethought algebraically in O(|a| + |b| + 26) instead of the original's O(26^3)
// mutate-recompute-restore triple loop: replacing present letter i with j adds
// acnt[j] - acnt[i], so the best gain is max(0, max(acnt) - min(acnt[i] | bcnt[i] > 0)).

let (n, m): (1..=1e6, 1..=1e6) = read()
let (a, b): (Str, Str) = read()

// FRICTION MC9: Character-indexed domains (`domain Letter = 'A'..='Z'`) work in
// `proto/cpi` and eliminate `- 'A'` noise, yet the cheat sheet only mentions integer
// domains (`1..=n`), leaving `'A'..='Z'` domains and `Letter(ch)` casts unspecified.
domain Letter = 'A'..='Z'
var acnt: [Letter]Int = 0
var bcnt: [Letter]Int = 0
for ch in a: acnt[Letter(ch)] += 1
for ch in b: bcnt[Letter(ch)] += 1

let base = sum(acnt[c] * bcnt[c] for c in Letter)
let hi = max(acnt[c] for c in Letter) ?? 0
let lo = min(acnt[c] for c in Letter if bcnt[c] > 0)
// FRICTION MC12: Cheat sheet specifies `if c: a else: b` as an expression and `if let x = o:`
// as a statement, leaving `if let x = o: a else: b` in expression position unspecified (and
// rejected by `proto/cpi`), with no `?T.map(...)` helper to bridge the gap.
let gain = if let x = lo: max(0, hi - x) else: 0

print(base + gain)
