// Source: Mock CCC '24 Contest 1 S3 (Optimal Split Point), https://dmoj.ca/problem/mccc6s3
// Problem: n <= 1e6 values a_i in 1..=1e6. Pick a split index i (a_i belongs to neither side)
//   and optionally move one element j across, minimising |left sum - right sum|. Print i and
//   j (-1 for no move), lexicographically smallest with -1 before any index.
// Solution: sweep i left to right; when the right side is heavier by diff, the best element
//   to move left is the value nearest diff/2 still on the right: binary search over the
//   compressed values, skipping values with no occurrence left via DSU next/prev pointers,
//   earliest occurrence. A sweep over the reversed array covers moves to the right.
//   O(n log n).
// original: corpus/submissions/6103125.cpp

let n: Int = read()
domain Pos = 1..=n
let a: [Pos]Int = read()

var best: ?(d: Int, i1: Int, i2: Int) = None

fn consider(d: Int, i1: Int, i2: Int):
    if best.is_none_or(b => (d, i1, i2) < (b.d, b.i1, b.i2)):
        best = (d:, i1:, i2:)

// Compress distinct values once so DSU jump pointers work for arbitrary 64-bit values.
var all_v: List<Int> = [a[p] for p in Pos]
let sv = all_v.sorted()
var vals: List<Int> = []
for x in sv:
    if vals.len == 0 or vals.last! != x: vals.push(x)
let m = vals.len
domain U = 0..m

fn lb(target: Int) -> Int:
    let lo = (0..m).first_true(mid => vals[mid] >= target) ?? m
    lo

fn sweep(arr: [Pos]Int, prefer_first: Bool, is_rev: Bool):
    var pl: [U]Int = [U] u => u.to_idx
    var pr: [U]Int = [U] u => u.to_idx
    var occ: [U]List<Int> = [U] _ => []
    var head: [U]Int = [U] _ => 0

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

    var sum_r = 0
    for p in Pos:
        let x = arr[p]
        let u = U(lb(x))
        occ[u].push(p.to_idx)
        sum_r += x

    var sum_l = 0
    for p in Pos:
        let i = p.to_idx
        let x = arr[p]
        let u = U(lb(x))
        sum_r -= x
        head[u] += 1
        if head[u] == occ[u].len:
            pl[u] = u.to_idx - 1
            pr[u] = u.to_idx + 1

        if not is_rev:
            consider(abs(sum_l - sum_r), i, -1)

        let diff = sum_r - sum_l
        if diff >= 0:
            let c1 = find_r(lb((diff + 1) / 2))
            if c1 < m:
                let uc = U(c1)
                let rep = if prefer_first: occ[uc][head[uc]] else: occ[uc][occ[uc].len - 1]
                let oi1 = if is_rev: n - 1 - i else: i
                let oi2 = if is_rev: n - 1 - rep else: rep
                consider(abs(diff - 2 * vals[c1]), oi1, oi2)

            let k_le = lb(diff / 2 + 1) - 1
            let c2 = find_l(k_le)
            if c2 >= 0:
                let uc = U(c2)
                let rep = if prefer_first: occ[uc][head[uc]] else: occ[uc][occ[uc].len - 1]
                let oi1 = if is_rev: n - 1 - i else: i
                let oi2 = if is_rev: n - 1 - rep else: rep
                consider(abs(diff - 2 * vals[c2]), oi1, oi2)

        sum_l += x

sweep(a, true, false)
let rev_a: [Pos]Int = [Pos] p => a[Pos.nth(n - 1 - p.to_idx)]
sweep(rev_a, false, true)

let b = best ?? (d: 0, i1: 0, i2: -1)
let out2 = if b.i2 < 0: -1 else: b.i2 + 1
print(b.i1 + 1, out2)
