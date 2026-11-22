// IOI '14 P6 (Holiday): maximize attraction sum in d days starting at city `start`.
// Split into walking one side twice (going and returning) plus the other side once,
// with O(1) optional start-city visit. Each side uses monotone divide-and-conquer DP
// over sqrt-bucket prefix top-k sums.
// original: corpus/submissions/4691224.cpp

use std.algebra.{desc}
use std.seq.{reverse}

const B = 300

struct TopK:
    sorted_pref: List<List<Int>>
    psum: List<List<Int>>
    gaps: List<List<(Int, Int)>>

    fn new(a: List<Int>) -> TopK:
        var sorted_pref: List<List<Int>> = []
        var psum: List<List<Int>> = []
        var gaps: List<List<(Int, Int)>> = []
        for ind in (0..=a.len).step(B):
            let s = a[..ind].sorted(by: desc)
            sorted_pref.push(s)
            let ps = scan(s, +)
            psum.push([ps.prefix(k) for k in 0..=s.len])
            let hi = min(a.len, ind + B)
            gaps.push([(a[j], j) for j in ind..hi].sorted(by: desc))
        TopK(sorted_pref, psum, gaps)

    fn query(self, i: Int, k: Int) -> Int:
        if k <= 0: return 0
        var rem = min(k, i + 1)
        let b = i / B
        var ans = 0
        for (x, j) in self.gaps[b]:
            if rem == 0: break
            if j <= i:
                if rem - 1 >= self.sorted_pref[b].len or x > self.sorted_pref[b][rem - 1]:
                    rem -= 1
                    ans += x
                else:
                    break
        ans + self.psum[b][rem]

fn solve_side(mul: Int, d: Int, a: List<Int>) -> List<Int>:
    var ans = List.filled(d + 1, 0)
    if a.is_empty: return ans
    let ds = TopK.new(a)
    fn rec(l: Int, r: Int, ql: Int, qr: Int):
        if l > r: return
        let mid = (l + r) / 2
        var best = -1
        var opt = ql
        for i in ql..=qr:
            let v = ds.query(i, mid - (i + 1) * mul)
            if v > best:
                best = v
                opt = i
        ans[mid] = best
        rec(l, mid - 1, ql, opt)
        rec(mid + 1, r, opt, qr)
    rec(0, d, 0, a.len - 1)
    ans

fn one_way(start: Int, d: Int, a: List<Int>) -> Int:
    let right = solve_side(2, d, a[start + 1..].to_list())
    var left_a = a[..start].to_list()
    reverse(mut left_a)
    let left = solve_side(1, d, left_a)
    var best = 0
    for take_start in 0..=1:
        for j in 0..=d:
            let k = d - j - take_start
            if k >= 0: best max= take_start * a[start] + right[j] + left[k]
    best

let (n, start, d): (Int, Int, Int) = read()
let a: List<Int> = [read() for _ in 0..n]
var rev_a = a.copy()
reverse(mut rev_a)
print(max(one_way(start, d, a), one_way(n - 1 - start, d, rev_a)))
