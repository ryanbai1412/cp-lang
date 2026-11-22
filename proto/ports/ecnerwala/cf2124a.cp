// Codeforces 2124 A — Deranged Deletions
//
// A subsequence is a derangement of its sorted order iff no element stays at
// its sorted position. If `a` is non-decreasing, every subsequence is already
// sorted so no derangement exists. Otherwise some adjacent pair has `x > y`,
// and `[x, y]` itself differs from its sorted order `[y, x]` at both positions.

@!multitest
let n: 1..=100 = read()
let a: [0..n]1..=100 = read()
for (x, y) in a.adjacent if x > y:
    print("YES")
    print(2)
    print(x, y)
    return
print("NO")
