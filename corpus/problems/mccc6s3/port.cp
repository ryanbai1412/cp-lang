// Source: Mock CCC '24 Contest 1 S3 (Optimal Split Point), https://dmoj.ca/problem/mccc6s3
// limits: 1000 ms, 512 MB
// Problem: n <= 1e6 values a_i in 1..=1e6. Pick a split index i (a_i belongs to neither side)
//   and optionally move one element j across, minimising |left sum - right sum|. Print i and
//   j (-1 for no move), lexicographically smallest with -1 before any index.
// Solution: sweep the split left to right. Moving a value x from the heavier right side
//   changes the gap by 2x, so the best x is the right-side value nearest half the gap: two
//   path-halving skip lists over the values (next present above, previous present below)
//   drop a value when its last occurrence leaves the right. The same sweep read backwards
//   covers moves to the right. O(n α(n)).
// Bounds: n: 1..=1e6; a: 1..=1e6
// original: corpus/submissions/6103125.cpp
let n: 1..=1_000_000 = read()
domain Pos = 0..n
let a: [Pos]1..=1_000_000 = read()
let m = a.max()! + 1
domain U = 0..m

// the answer as printed: the gap, the 1-based split, the 1-based move or -1 for none (the
// statement orders answers by these)
var best = (d: Int.MAX, at: 0, moved: 0)

fn consider(d: Int, at: Pos, moved: Int):
    if (d, Int(at) + 1, moved) < (best.d, best.at, best.moved): best = (d:, at: Int(at) + 1, moved:)

fn sweep(rev: Bool):
    fn back(p: Pos) -> Pos = if rev: Pos(n - 1 - Int(p)) else: p
    fn arr(p: Pos) -> Int = a[back(p)]
    var last: [U]?Pos = None
    var first: [U]?Pos = None
    var after: [Pos]?Pos = None
    for p in Pos.rev:
        let u = U(arr(p))
        after[p] = first[u]
        first[u] = Some(p)
        if last[u] == None: last[u] = Some(p)
    // skip lists over the values: `nxt[u]` leads to the next present value >= u (m: none),
    // `prv[u + 1]` to one plus the previous present value <= u (0: none); both halve paths
    var nxt = [0..=m] u => if u < m and last[U(u)] != None: u else: u + 1
    var prv = [0..=m] i => if i > 0 and last[U(i - 1)] != None: i else: i - 1
    nxt[m] = m
    prv[0] = 0
    fn above(u: Int) -> Int:
        var x = u
        while nxt[x] != x:
            nxt[x] = nxt[nxt[x]]
            x = nxt[x]
        x
    fn below(u: Int) -> Int:
        var x = u + 1
        while prv[x] != x:
            prv[x] = prv[prv[x]]
            x = prv[x]
        x - 1
    fn drop(u: U):
        nxt[Int(u)] = Int(u) + 1
        prv[Int(u) + 1] = Int(u)

    fn take(c: U, diff: Int, p: Pos):
        let j = if rev: last[c]! else: first[c]!
        consider(abs(diff - 2 * Int(c)), back(p), Int(back(j)) + 1)

    var right = a.sum()
    var left = 0
    for p in Pos:
        let u = U(arr(p))
        right -= arr(p)
        if not rev: first[u] = after[p]
        if last[u] == Some(p): drop(u)
        if not rev: consider(abs(left - right), p, -1)
        let diff = right - left
        if diff >= 0:
            if let x = U.at(above(min((diff + 1) / 2, m))): take(x, diff, p)
            if let x = U.at(below(min(diff / 2, m - 1))): take(x, diff, p)
        left += arr(p)

sweep(false)
sweep(true)

print(best.at, best.moved)
