// Codeforces 2089C2 (Key of Like, hard), tuned.
// port.cp's dp with no list built per round: one prefix array over the
// rotated row is reused for both windows, the cyclic window and the fake-key
// shift index without `%` in the loop, and the row is written in place.
type Z = Mod<1_000_000_007>
let comb = Comb<Z>()

@!multitest
let n: 1..=100 = read()
let l: 1..=5_000 = read()
let k_max: 0..=25 = read()

var dp: [0..=k_max][0..n]Z = 0
var pref: [0..=n]Z = 0
var next_row: [0..n]Z = 0

// Sums the rotated row over the `len` positions ending at `j`, cyclically.
fn window(len: Int, j: Int) -> Z:
    let rem = len % n
    let tail = if rem <= j + 1: pref[j + 1] - pref[j + 1 - rem] else: pref[j + 1] + pref[n] - pref[2 * n - rem + j + 1 - n]
    pref[n] * (len / n) + tail

for i in 1..=l + k_max:
    let inv_i = comb.inv(i)
    for k in (0..=min(k_max, i)).rev:
        let (p_lock, p_key): (Z, Z) = if i == 1 or k == i:
            (1, 0)
        else:
            let denom = comb.inv(2 * i - k - 2)
            ((i - 1) * denom, (i - k - 1) * denom)
        if k < i:
            pref[1] = dp[k][n - 1] + 1
            for j in 1..n: pref[j + 1] = pref[j] + dp[k][j - 1]
            let (c_lock, c_key) = (inv_i * p_lock, inv_i * p_key)
            for j in 0..n: next_row[j] = c_lock * window(i, j) + c_key * window(i - k, j)
        else:
            for j in 0..n: next_row[j] = 0
        if k > 0:
            let c_fake = k * inv_i * p_key
            let s = (i - k) % n
            for j in 0..n:
                let src = if j >= s: j - s else: j - s + n
                next_row[j] += c_fake * dp[k - 1][src]
        for j in 0..n: dp[k][j] = next_row[j]

print(dp[k_max])
