// Codeforces 2053 I1: the shortest b holding a as a subsequence, with a's sum s and boredom s.
// Boredom s is the floor, and it holds exactly when every prefix sum of b lies in [0, s].
// Scan a keeping the interval of prefix sums still reachable; when it empties, one inserted
// element before a_i resets it to all of [0, s]. b must end at s, which may cost one more.
@!multitest
domain N = read(1..=3e6)
let a = read<[N]-1e9..=1e9>()
let s = a.sum()
let bounds = 0..=s

var reach = 0..=0
var inserted = 0
for x in a:
    reach = (reach.start + x..=reach.last + x).clip(bounds)
    if reach.is_empty:
        inserted += 1
        reach = (x..=s + x).clip(bounds)
print(a.len + inserted + Int(s not in reach))
