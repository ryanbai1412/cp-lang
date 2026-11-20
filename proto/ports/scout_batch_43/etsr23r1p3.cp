// ETSR '23 Round 1 P3 — All-Permutations Subsequence Bitmask DP (CP Lang v2 spec)
// original: corpus/submissions/5494303.cpp
// spec: docs/one-page-cheat-sheet.md (3723dca)
//
// Decide whether every permutation of the first `m` lowercase letters appears
// as a subsequence of `s`. Uses backward character-occurrence automaton `prv`
// and subset DP over `Subset<Alpha>`.

@multitest
let (m, s): (1..=26, Str) = read()
if m >= 22:
    print("NO")
    return

let n = s.len
domain Alpha = 0..m
var prv: [0..=n, Alpha]?Int = none
for i in 0..n:
    for c in Alpha:
        prv[i + 1, c] = prv[i, c]
    prv[i + 1, Alpha(s[i] - 'a')] = i

// FRICTION ET1: `min=` on `?Int` skips `none` (treating `none` as +inf),
// whereas here `none` is an absorbing failure (if any submask fails, the
// whole mask fails). Requires explicit `if let ... else: best = none; break`.
var dp: [Subset<Alpha>]?Int = none
dp[Subset<Alpha>.first] = n

for mask in Subset<Alpha> if mask.len > 0:
    var best: ?Int = n
    for c in Alpha if c in mask:
        if let prev_pos = dp[mask.without(c)] and let hit = prv[prev_pos, c]:
            best min= hit
        else:
            best = none
            break
    dp[mask] = best

print(if dp[Subset<Alpha>.last] != none: "YES" else: "NO")
