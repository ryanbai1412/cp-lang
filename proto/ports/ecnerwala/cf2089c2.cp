const MOD = 1_000_000_007

@! multitest
let n: Int = read()
let l: Int = read()
let k_max: Int = read()
var pref: [0..=n]Mod<MOD> = 0
var dp: [0..=k_max][0..n]Mod<MOD> = 0

fn prefix_sum(ways: [0..n]Mod<MOD>, len: Int) -> [0..n]Mod<MOD>:
    pref[0] = 0
    for j in 0..n:
        pref[j + 1] = pref[j] + ways[j]
    var res: [0..n]Mod<MOD> = 0
    for j in 0..n:
        let a = j + 1 - len
        let cnt = (n - a) / n
        let p = pref[a + n * cnt] - pref[n] * cnt
        res[j] = pref[j + 1] - p
    res

for i in 1..=l + k_max:
    var k = min(k_max, i)
    let inv_i = (Mod<MOD>(i)).inv()
    while k >= 0:
        var r: [0..n]Mod<MOD> = 0
        var p_lock: Mod<MOD> = i - 1
        var p_key: Mod<MOD> = i - k - 1
        if i == 1 or k == i:
            p_lock = 1
            p_key = 0
        else:
            let d = (p_lock + p_key).inv()
            p_lock *= d
            p_key *= d
        if k < i:
            var v: [0..n]Mod<MOD> = 0
            v[0] = dp[k][n - 1] + 1
            for j in 1..n:
                v[j] = dp[k][j - 1]
            let v1 = prefix_sum(v, i)
            let c1 = inv_i * p_lock
            for j in 0..n:
                r[j] += c1 * v1[j]
            let v2 = prefix_sum(v, i - k)
            let c2 = inv_i * p_key
            for j in 0..n:
                r[j] += c2 * v2[j]
        if k > 0:
            let c = Mod<MOD>(k) * inv_i * p_key
            let off = (i - k) % n
            for j in 0..n:
                r[j] += c * dp[k - 1][(j + n - off) % n]
        for j in 0..n:
            dp[k][j] = r[j]
        k -= 1
let out: List<Mod<MOD>> = [dp[k_max][j] for j in 0..n]
print(out)
