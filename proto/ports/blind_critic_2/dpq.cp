// DP Contest Q (Flowers): max total beauty of a subsequence with increasing heights.
let n: 1..=2e5 = read()
domain I = 1..=n
let h: [I]I = read() where distinct(h)
let a: [I]1..=1e9 = read()

@solution
fn main():
    var best = SegTree<I, Int>(0, (x, y) => max(x, y))
    for i in I:
        var below = 0
        if let p = h[i] - 1: below = best.query(I.first, p)
        best.set(h[i], a[i] + below)
    print(best.query(I.first, I.last))

@brute(main)
fn slow():
    var best: [I]Int = 0
    for i in I:
        best[i] = a[i] + (max(best[j] for j in I.first..i if h[j] < h[i]) ?? 0)
    print(max(best))
