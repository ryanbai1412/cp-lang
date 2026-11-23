// Mock CCC '22 Contest 1 S1 (nccc9s1), buildable under proto/cpc today. See ../nccc9s1.cp.
// original: corpus/submissions/4051682.cpp
@!multitest
let a: 0..=1_000_000_000 = read()
let b: 0..=1_000_000_000 = read()
let c: 0..=1_000_000_000 = read()
let ok = (a + 2 * b + 3 * c) % 2 == 0 and (if c > 0: a >= c else: b % 2 == 0 or a >= 2)
if ok: print("YES")
else: print("NO")
