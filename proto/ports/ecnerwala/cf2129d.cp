const VMAX = 10
type Z = Mod<998_244_353>
let comb = Comb<Z.MOD>()

@!multitest
let n: Int = read()
let raw: [0..n]Int = read()
let s: [0..n]?Int = [0..n] i => if raw[i] < 0: none else: raw[i]

var dp: [0..=n, 0..=n, 0..=VMAX, 0..=VMAX]Z = 0
for i in 0..=n: dp[i, i, 0, 0] = 1

for len in 1..=n:
    let w = Z(len).inv
    for i in 0..=n - len:
        let j = i + len - 1
        let sl = if i > 0: s[i - 1] else: none
        let sr = if j + 1 < n: s[j + 1] else: none
        let vi_max = min(sl ?? 0, VMAX)
        let vj_max = min(sr ?? 0, VMAX)
        for k in i..=j:
            let ai = Int(sl != none and (j + 1 >= n or k - i <= j - k))
            let aj = Int(sr != none and (i == 0 or j - k < k - i))
            let sk = s[k]
            let vk_max = min(sk ?? 0, VMAX)
            for vi in 0..=vi_max:
                for vkl in 0..=vk_max:
                    let left = dp[i, k, vi, vkl]
                    if left == 0: continue
                    let vkr = if let c = sk: c - vkl else: 0
                    if vkr > VMAX: continue
                    let lw = left * w
                    for vj in 0..=vj_max:
                        let r = dp[k + 1, j + 1, vkr, vj]
                        if r != 0:
                            dp[i, j + 1, vi + ai, vj + aj] += lw * r

print(dp[0, n, 0, 0] * comb.fact(n))
