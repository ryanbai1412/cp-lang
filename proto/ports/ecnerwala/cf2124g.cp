@!multitest
let n: Int = read()
let a: [0..n]Int = read()
var mins: [0..n]Int = a
for i in 1..n: mins[i] min= mins[i - 1]

let base_ans = mins.sum()
var ans: [0..n]Int = base_ans
var best_rem: List<(val: Int, pos: Int)> = []
var suff_val: [0..=n]Int = 0
var j = n

for i in (0..n).rev:
    let ai = a[i]
    if mins[i] == ai:
        let cap = if i == 0: ai + best_rem.last!.val else: mins[i - 1]
        for k in i..j:
            mins[k] = if k == i: cap else: min(mins[k - 1], a[k])
        for k in (i..j).rev:
            suff_val[k] = suff_val[k + 1] + mins[k]
        var k = j
        var tot_gain = 0
        for (v_incr, pos_incr) in best_rem:
            let cur_cap = min(cap, ai + v_incr)
            while k > i and mins[k - 1] <= cur_cap:
                k -= 1
                tot_gain += mins[k] - ai
            ans[pos_incr - i] max= base_ans + (cur_cap - ai) * (k - i) + tot_gain - suff_val[pos_incr]
            if cur_cap >= cap: break
        for k in (i..j).rev:
            mins[k] = ai
            suff_val[k] = suff_val[k + 1] + ai
        j = i
    if best_rem.is_empty or ai > best_rem.last!.val:
        best_rem.push((ai, i))

for i in (1..n).rev: ans[i - 1] max= ans[i]
print(ans)
