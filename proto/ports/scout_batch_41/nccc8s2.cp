// Mock CCC '21 S2 (nccc8s2): count subsequences of s with all distinct letters
// (including the empty subsequence), modulo 10^9 + 7.
// original: corpus/submissions/3381135.java
type Z = Mod<1_000_000_007>
let s: Str = read()
var cnt: ['a'..='z']Int = 0
for c in s: cnt[c] += 1
var ans: Z = 1
for x in cnt: ans *= x + 1
print(ans)
