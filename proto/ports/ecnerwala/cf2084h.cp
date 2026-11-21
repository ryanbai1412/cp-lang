@!multitest
type M = Mod<1_000_000_007>

let n: 3..=2_000_000 = read()
let s: Str = read()

var runs = List<Int>()
var run = 1
for i in 1..n:
    if s[i] == s[i - 1]:
        run += 1
    else:
        runs.push(run)
        run = 1
runs.push(run)

if runs.len == 1:
    print(M(n - 1))
else:
    var offset: [0..2]Int = 0
    var stk: [0..2]List<(len: Int, val: M)> = []
    var stk_sum: [0..2]M = 0
    var prev_dp: M = 1

    for k in 0..runs.len - 1:
        let z = k % 2
        offset[z] += 1
        let shifted = runs[k] - offset[z]

        if k >= 1:
            while let top = stk[z].last, shifted >= top.len:
                stk[z].pop()
                if let below = stk[z].last:
                    stk_sum[z] -= M(below.len - top.len) * below.val
            if let below = stk[z].last:
                stk_sum[z] += M(below.len - shifted) * below.val
            stk[z].push((len: shifted, val: prev_dp))

        var cur_dp = stk_sum[z]
        if let top = stk[z].last:
            cur_dp += M(top.len + offset[z]) * top.val
        let base = if z == 0: runs[0] else: 1
        cur_dp += M(base + offset[z] - 1)
        prev_dp = cur_dp

    print(prev_dp * M(runs.last!))
