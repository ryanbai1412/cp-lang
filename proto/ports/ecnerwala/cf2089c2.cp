const MOD = 1_000_000_007
type Z = Mod<MOD>
let comb = Comb<Z>()

@!multitest
let (n, l, k_max): (Int, Int, Int) = read()
var pref: [0..=n]Z = 0
var dp: [0..=k_max, 0..n]Z = 0

fn prefix_sum(ways: [0..n]Z, len: Int) -> [0..n]Z:
    pref[0] = 0
    for j in 0..n: pref[j + 1] = pref[j] + ways[j]
    var res: [0..n]Z = 0
    for j in 0..n:
        let a = j + 1 - len
        let cnt = (n - a) / n
        res[j] = pref[j + 1] - (pref[a + n * cnt] - pref[n] * cnt)
    res

for i in 1..=l + k_max:
    let inv_i = comb.inv(i)
    for k in (0..=min(k_max, i)).rev:
        var r: [0..n]Z = 0
        let (p_lock, p_key): (Z, Z) = if i == 1 or k == i:
            (1, 0)
        else:
            let d = comb.inv(2 * i - k - 2)
            ((i - 1) * d, (i - k - 1) * d)
        if k < i:
            let v = [0..n] j => if j == 0: dp[k, n - 1] + 1 else: dp[k, j - 1]
            let v1 = prefix_sum(v, i)
            let v2 = prefix_sum(v, i - k)
            let (c1, c2) = (inv_i * p_lock, inv_i * p_key)
            for j in 0..n: r[j] += c1 * v1[j] + c2 * v2[j]
        if k > 0:
            let c = k * inv_i * p_key
            let off = (i - k) % n
            for j in 0..n: r[j] += c * dp[k - 1, (j + n - off) % n]
        for j in 0..n: dp[k, j] = r[j]

print([dp[k_max, j] for j in 0..n])
