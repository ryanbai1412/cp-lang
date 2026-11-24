// Source: Mock CCC '24 Contest 1 S3 (Optimal Split Point), https://dmoj.ca/problem/mccc6s3
// limits: 1000 ms, 512 MB
// Problem: n <= 1e6 values a_i in 1..=1e6. Pick a split index i (a_i belongs to neither side)
//   and optionally move one element j across, minimising |left sum - right sum|. Print i and
//   j (-1 for no move), lexicographically smallest with -1 before any index.
// Solution: sweep i left to right; when the right side is heavier by diff, the best element
//   to move left is the value nearest diff/2 still on the right, found by DSU next/prev
//   pointers over the values that skip those with no occurrence left, earliest occurrence. A sweep over the reversed array covers moves to the right.
//   O(n log n).
// original: corpus/submissions/6103125.cpp

let n: Int = read()
domain Pos = 1..=n
let a: [Pos]Int = read()

var best: ?(d: Int, i1: Int, i2: Int) = None

fn consider(d: Int, i1: Int, i2: Int):
    if best.is_none_or(b => (d, i1, i2) < (b.d, b.i1, b.i2)):
        best = (d:, i1:, i2:)

let m = a.max()! + 1
domain U = 0..m

// the first value >= target, or m
fn lb(target: Int) -> Int = max(0, min(target, m))

fn sweep(arr: [Pos]Int, prefer_first: Bool, is_rev: Bool):
    // rep[u]: the occurrence of value u right of the split that a move takes, the first
    // (prefer_first) or the last; next[p]: the occurrence of arr[p] after p
    var rep: [U]?Pos = None
    var next: [Pos]?Pos = None
    for p in Pos.rev:
        let u = U(arr[p])
        next[p] = rep[u]
        if prefer_first or rep[u] == None: rep[u] = Some(p)
    var pl: [U]Int = [U] u => if rep[u] == None: u.ord - 1 else: u.ord
    var pr: [U]Int = [U] u => if rep[u] == None: u.ord + 1 else: u.ord

    fn find_l(x: Int) -> Int:
        if x < 0: return -1
        let u = U(x)
        if pl[u] == x: return x
        let r = find_l(pl[u])
        pl[u] = r
        r

    fn find_r(x: Int) -> Int:
        if x >= m: return m
        let u = U(x)
        if pr[u] == x: return x
        let r = find_r(pr[u])
        pr[u] = r
        r

    fn take(c: Int, diff: Int, i: Int):
        let j = rep[U(c)]!.ord
        if is_rev: consider(abs(diff - 2 * c), n - 1 - i, n - 1 - j)
        else: consider(abs(diff - 2 * c), i, j)

    var sum_r = arr.sum()
    var sum_l = 0
    for p in Pos:
        let i = p.ord
        let x = arr[p]
        let u = U(x)
        sum_r -= x
        if prefer_first: rep[u] = next[p]
        elif rep[u] == Some(p): rep[u] = None
        if rep[u] == None:
            pl[u] = u.ord - 1
            pr[u] = u.ord + 1

        if not is_rev: consider(abs(sum_l - sum_r), i, -1)

        let diff = sum_r - sum_l
        if diff >= 0:
            let c1 = find_r(lb((diff + 1) / 2))
            if c1 < m: take(c1, diff, i)
            let c2 = find_l(lb(diff / 2 + 1) - 1)
            if c2 >= 0: take(c2, diff, i)

        sum_l += x

sweep(a, true, false)
let rev_a: [Pos]Int = [Pos] p => a[Pos.nth(n - 1 - p.ord)]
sweep(rev_a, false, true)

let ans = best ?? (d: 0, i1: 0, i2: -1)
print(ans.i1 + 1, if ans.i2 < 0: -1 else: ans.i2 + 1)
