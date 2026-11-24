// Source: Mock CCC '24 Contest 1 S3 (Optimal Split Point), https://dmoj.ca/problem/mccc6s3
// limits: 1000 ms, 512 MB
// Problem: n <= 1e6 values a_i in 1..=1e6. Pick a split index i (a_i belongs to neither side)
//   and optionally move one element j across, minimising |left sum - right sum|. Print i and
//   j (-1 for no move), lexicographically smallest with -1 before any index.
// Solution: sweep the split left to right. Moving a value x from the heavier right side
//   changes the gap by 2x, so the best x is the right-side value nearest half the gap: two
//   `DsuOf` skip lists over the values (next present above, previous present below) drop a
//   value when its last occurrence leaves the right. The same sweep over the reversed array
//   covers moves to the right. O(n α(n)).
// Bounds: n: 1..=1e6; a: 1..=1e6
// original: corpus/submissions/6103125.cpp
let n: 1..=1_000_000 = read()
domain Pos = 0..n
let a: [Pos]1..=1_000_000 = read()
let m = a.max()! + 1
domain U = 0..m

var best: ?(d: Int, at: Pos, moved: ?Pos) = None

// the printed move: its 1-based index, or -1 for none (the statement orders answers by this)
fn shown(moved: ?Pos) -> Int = if let j = moved: Int(j) + 1 else: -1

fn consider(d: Int, at: Pos, moved: ?Pos):
    if best.is_none_or(b => (d, at, shown(moved)) < (b.d, b.at, shown(b.moved))):
        best = (d:, at:, moved:)

fn sweep(arr: [Pos]Int, rev: Bool):
    fn back(p: Pos) -> Pos = if rev: Pos(n - 1 - Int(p)) else: p
    var last: [U]?Pos = None
    var first: [U]?Pos = None
    var after: [Pos]?Pos = None
    for p in Pos.rev:
        let u = U(arr[p])
        after[p] = first[u]
        first[u] = Some(p)
        if last[u] == None: last[u] = Some(p)
    var up = DsuOf<U, ?U>(init: u => if last[u] != None: Some(u) else: None, join: (_, b) => b)
    var down = DsuOf<U, ?U>(init: u => if last[u] != None: Some(u) else: None, join: (_, b) => b)
    fn drop(u: U):
        up.set(u, None)
        if let v = U.at(Int(u) + 1): up.unite(u, v)
        down.set(u, None)
        if let v = U.at(Int(u) - 1): down.unite(u, v)
    for u in U if last[u] == None: drop(u)

    fn take(c: U, diff: Int, p: Pos):
        let j = if rev: last[c]! else: first[c]!
        consider(abs(diff - 2 * Int(c)), back(p), Some(back(j)))

    var right = arr.sum()
    var left = 0
    for p in Pos:
        let u = U(arr[p])
        right -= arr[p]
        if not rev: first[u] = after[p]
        if last[u] == Some(p): drop(u)
        if not rev: consider(abs(left - right), p, None)
        let diff = right - left
        if diff >= 0:
            if let c = U.at((diff + 1) / 2) and let x = up.get(c): take(x, diff, p)
            if let x = down.get(U(min(diff / 2, m - 1))): take(x, diff, p)
        left += arr[p]

sweep(a, false)
sweep([Pos] p => a[Pos(n - 1 - Int(p))], true)

let win = best!
print(Int(win.at) + 1, shown(win.moved))
