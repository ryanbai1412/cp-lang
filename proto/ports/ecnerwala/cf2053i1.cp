// Codeforces 2053 I1: the shortest b holding a as a subsequence, with a's sum s and boredom s.
// Boredom s is the floor, and it holds exactly when every prefix sum of b lies in [0, s].
// Scan a keeping the interval of prefix sums still reachable; when it empties, one inserted
// element before a_i resets it to all of [0, s]. b must end at s, which may cost one more.
@!multitest
let n: 1..=3e6 = read()
let a: [0..n]-1e9..=1e9 = read()
let s = a.sum()

var reach = 0..=0
var inserted = 0
for x in a:
    reach = max(reach.lo + x, 0)..=min(reach.last + x, s)
    if reach.is_empty:
        inserted += 1
        reach = max(x, 0)..=min(s + x, s)
print(n + inserted + Int(s not in reach))
