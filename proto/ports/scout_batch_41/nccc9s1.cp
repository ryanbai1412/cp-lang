// Mock CCC '22 Contest 1 S1 (nccc9s1): decide whether a 1s, b 2s and c 3s can
// be split into two equal-sum groups under the contest pairing rules.
// original: corpus/submissions/4051682.cpp
@multitest
let (a, b, c): (0..=1_000_000_000, 0..=1_000_000_000, 0..=1_000_000_000) = read()
let ok = (a + 2 * b + 3 * c) % 2 == 0 and (if c > 0: a >= c else: b % 2 == 0 or a >= 2)
print(if ok: "YES" else: "NO")
