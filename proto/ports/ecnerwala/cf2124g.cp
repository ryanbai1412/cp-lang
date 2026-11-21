@!multitest
let n: Int = read()
let a: List<Int> = [read::<Int>() for _ in 0..n]
var mins = a
for i in 1..n:
    mins[i] min= mins[i - 1]
let base_ans = sum(x for x in mins)
var ans = List.filled(n, base_ans)
var best_rem: List<(Int, Int)> = []
var suff_val: [0..(n + 1)]Int = 0
var j = n
for i in (0..n).rev:
    if mins[i] == a[i]:
        let cap = if i == 0: a[0] + best_rem.last()!.0 else: mins[i - 1]
        for k in i..j:
            mins[k] = if k == i: cap else: min(mins[k - 1], a[k])
        for k in (i..j).rev:
            suff_val[k] = suff_val[k + 1] + mins[k]
        var k = j
        var tot_gain = 0
        for (v_incr, pos_incr) in best_rem:
            let cur_cap = min(cap, a[i] + v_incr)
            while k > i and mins[k - 1] <= cur_cap:
                k -= 1
                tot_gain += mins[k] - a[i]
            let cnd = base_ans + (cur_cap - a[i]) * (k - i) + tot_gain - suff_val[pos_incr]
            ans[pos_incr - i] max= cnd
            if cur_cap >= cap: break
        for k in (i..j).rev:
            mins[k] = a[i]
            suff_val[k] = suff_val[k + 1] + mins[k]
        j = i
    if best_rem.is_empty or a[i] > best_rem.last()!.0:
        best_rem.push((a[i], i))
for i in (1..n).rev:
    ans[i - 1] max= ans[i]
print(ans)
