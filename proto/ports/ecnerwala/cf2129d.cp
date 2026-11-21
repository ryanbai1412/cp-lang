type Z = Mod<998244353>

@!multitest
let n: Int = read()
let s: List<Int> = [read::<Int>() for _ in 0..n]
let vmax = 10
let vs = vmax + 1

var fact: [0..=n]Z = 1
for i in 1..=n: fact[i] = fact[i - 1] * i

let stride_j = vs * vs
let stride_i = (n + 1) * stride_j
var dp: [0..((n + 1) * stride_i)]Z = 0
fn idx(i: Int, j: Int, vi: Int, vj: Int) -> Int = i * stride_i + j * stride_j + vi * vs + vj

for i in 0..=n: dp[idx(i, i, 0, 0)] = 1
for len in 1..=n:
    let w = Z(len).inv()
    for i in 0..=(n - len):
        let j = i + len - 1
        let vi_max = if i == 0 or s[i - 1] < 0: 0 else: min(s[i - 1], vmax)
        let vj_max = if j + 1 >= n or s[j + 1] < 0: 0 else: min(s[j + 1], vmax)
        for k in i..=j:
            let ai = Int(i > 0 and s[i - 1] >= 0 and (j + 1 >= n or k - i <= j - k))
            let aj = Int(j + 1 < n and s[j + 1] >= 0 and (i == 0 or j - k < k - i))
            let vk_max = if s[k] < 0: 0 else: min(s[k], vmax)
            for vi in 0..=vi_max:
                for vkl in 0..=vk_max:
                    let left = dp[idx(i, k, vi, vkl)]
                    if left == 0: continue
                    let vkr = if s[k] < 0: 0 else: s[k] - vkl
                    if vkr > vmax: continue
                    let lw = left * w
                    for vj in 0..=vj_max:
                        let r = dp[idx(k + 1, j + 1, vkr, vj)]
                        if r != 0:
                            dp[idx(i, j + 1, vi + ai, vj + aj)] += lw * r
print(dp[idx(0, n, 0, 0)] * fact[n])
