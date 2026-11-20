// Codeforces 2245 F: 2D interval DP over expanded right-endpoint states,
// multiplying by inv(right_idx[k] - i) at each split and n! at the end.
const MOD = 998244353
type Z = Mod<MOD>
let comb = Comb<MOD>()

@!multitest
let n: Int = read()
var a: [0..=n]Int = -1
for i in 0..n: a[i] = read()

fn solve() -> Z:
    var tot = 0
    for i in 0..=n:
        if a[i] != -1: tot += a[i]
        if tot > i: return 0

    var right_idx: List<Int> = []
    var nxt_right: List<?Int> = []
    var first_right: [0..=n]Int = 0
    var last_right: [0..=n]Int = 0
    for i in 0..=n:
        first_right[i] = right_idx.len
        if a[i] == -1:
            right_idx.push(i)
            nxt_right.push(right_idx.len - 1)
        else:
            for _ in 0..a[i]:
                right_idx.push(i)
                nxt_right.push(right_idx.len)
            right_idx.push(i)
            nxt_right.push(none)
        last_right[i] = right_idx.len - 1

    let r = right_idx.len
    var dp: [0..=n, 0..r]Z = 0
    for i in (0..=n).rev:
        dp[i, first_right[i]] = 1
        for j in i..n:
            let v = dp[i, last_right[j]]
            if v == 0: continue
            for k in first_right[j + 1]..r:
                if let nk = nxt_right[k]:
                    dp[i, nk] += v * dp[j + 1, k] / (right_idx[k] - i)
    dp[0, last_right[n]] * comb.fact(n)

print(solve())
