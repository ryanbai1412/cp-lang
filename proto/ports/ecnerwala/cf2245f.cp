// Codeforces 2245 F: 2D interval DP over expanded right-endpoint states,
// multiplying by inv(right_idx[k] - i) at each split and n! at the end.
type Z = Mod<998_244_353>
let comb = Comb<Z>()

@!multitest
let n: Int = read()
let raw_a: [0..n]Int = read()
let a: [0..=n]?Int = [0..=n] i => if i == n or raw_a[i] < 0: none else: raw_a[i]

fn solve() -> Z:
    var tot = 0
    for i in 0..=n:
        if let v = a[i]: tot += v
        if tot > i: return 0

    var right_idx: [0..2 * n + 1]Int = 0
    var nxt_right: [0..2 * n + 1]?Int = none
    var first_right: [0..=n]Int = 0
    var last_right: [0..=n]Int = 0
    var r = 0
    for i in 0..=n:
        first_right[i] = r
        if let cnt = a[i]:
            for _ in 0..cnt:
                right_idx[r] = i
                nxt_right[r] = r + 1
                r += 1
            right_idx[r] = i
            nxt_right[r] = none
            r += 1
        else:
            right_idx[r] = i
            nxt_right[r] = r
            r += 1
        last_right[i] = r - 1

    var dp: [0..=n, 0..r]Z = 0
    for i in (0..=n).rev:
        dp[i, first_right[i]] = 1
        for j in i..n:
            let v = dp[i, last_right[j]]
            if v == 0: continue
            for k in first_right[j + 1]..r:
                if let nk = nxt_right[k]:
                    dp[i, nk] += v * dp[j + 1, k] * comb.inv(right_idx[k] - i)
    dp[0, last_right[n]] * comb.fact(n)

print(solve())
