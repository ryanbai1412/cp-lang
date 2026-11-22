// Codeforces 2053 I1: the shortest b holding a as a subsequence, with a's sum s and boredom s.
// Boredom s is the floor, and it holds exactly when every prefix sum of b lies in [0, s].
// Scan a keeping the interval of prefix sums still reachable; when it empties, one inserted
// element before a_i resets it to all of [0, s]. b must end at s, which may cost one more.
@!multitest
let a = read<[read(1..=3e6)]-1e9..=1e9>()
let s = a.sum()
let bounds = 0..=s

var reach = 0..=0
var inserted = 0
for x in a:
    reach = (reach + x) & bounds
    if reach.is_empty:
        inserted += 1
        reach = (bounds + x) & bounds
print(a.len + inserted + (s !in reach).int)
