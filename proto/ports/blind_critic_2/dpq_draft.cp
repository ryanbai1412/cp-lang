// DP Contest Q (Flowers): max total beauty of a subsequence with increasing heights.
// DRAFT 1: the obvious O(n^2) DP.
let n: 1..=2e5 = read()
domain I = 1..=n
let h: [I]I = read() where distinct(h)
let a: [I]1..=1e9 = read()

var best: [I]Int = 0
for i in I:
    best[i] = a[i] + (max(best[j] for j in I.first..i if h[j] < h[i]) ?? 0)
print(max(best))
