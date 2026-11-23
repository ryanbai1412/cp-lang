// Codeforces 2124 G: Maximise Sum, tuned.
// port.cp's algorithm with orig.cpp's layout: one `mins` row and one suffix
// sum row, rewritten in place over each prefix-minimum block and restored
// after it, instead of four new lists per block (`scan`, `suffix_fold` and
// two re-indexed copies), which cost a malloc each when every block is one
// element long. The candidate count `k` walks down the block once per block.

@!multitest
let n: 1..=1_000_000 = read()
let a: [0..n]0..=2_000_000 = read()

var mins: [0..n]0..=4_000_000 = [0..n] i => a[i]
for i in 1..n: mins[i] min= mins[i - 1]
let base_ans = mins.sum()
var ans: [0..n]Int = base_ans
var suff: [0..=n]Int = 0
// The strict suffix maxima seen so far, as (value, position), values rising.
var best: List<(0..=2_000_000, 0..n)> = []
var j = n
for i in (0..n).rev:
    let ai = a[i]
    if mins[i] == ai:
        let cap = if i == 0: ai + best.last!.0 else: mins[i - 1]
        mins[i] = cap
        for k in i + 1..j: mins[k] = min(mins[k - 1], a[k])
        for k in (i..j).rev: suff[k] = suff[k + 1] + mins[k]
        var k = j
        var gain = 0
        for (v, p) in best:
            let cur_cap = min(cap, ai + v)
            while k > i and mins[k - 1] <= cur_cap:
                k -= 1
                gain += mins[k] - ai
            ans[p - i] max= base_ans + (cur_cap - ai) * (k - i) + gain - suff[p]
            if cur_cap >= cap: break
        for k in (i..j).rev:
            mins[k] = ai
            suff[k] = suff[k + 1] + ai
        j = i
    if best.is_empty or ai > best.last!.0: best.push((ai, i))
for i in (1..n).rev: ans[i - 1] max= ans[i]
print(ans)
