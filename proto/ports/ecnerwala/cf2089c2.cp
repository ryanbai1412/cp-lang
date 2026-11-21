// Codeforces 2089 C2: Key of Like (Hard Version).
// `n` players take turns picking uniformly from `i` remaining locks, `k` of which
// are counterfeit (`i` grows from 1 up to `l + k_max` as locks are added back).
// `dp[k, j]` is the expected number of real locks opened by the player `j` steps
// after the current turn. For `i > 1` and `k < i`, the remaining `2i - k - 2`
// equally likely outcomes split into `i - 1` lock-first (`p_lock`) and `i - k - 1`
// key-first (`p_key`) transitions:
//   - Opening a real lock (`k < i`) scores +1 for the current player and rotates
//     the turn by 1..=i (weight `p_lock / i`) or 1..=i - k (weight `p_key / i`),
//     evaluated via `cyclic_window_sum`.
//   - Opening a counterfeit key (`k > 0`) drops `k -> k - 1` and rotates by `i - k`.
use std.prefix.prefix1

type Z = Mod<1_000_000_007>
let comb = Comb<Z>()

@!multitest
let n: 1..=100 = read()
let l: 1..=5_000 = read()
let k_max: 0..=100 = read()
domain Pos = 0..n

fn shift_back(p: Pos, off: Int) -> Pos = Pos((p.ord + n - off % n) % n)

// For each j in Pos, sums `ways[j - t mod n]` over t in 0..len.
fn cyclic_window_sum(ways: [Pos]Z, len: Int) -> [Pos]Z:
    let pref = prefix1(ways)
    let full = len / n
    let rem = len % n
    let total = pref.sum(Pos.first..=Pos.last)
    var res: [Pos]Z = 0
    for p in Pos:
        let j = p.ord
        let tail = if rem == 0:
            Z(0)
        elif j + 1 >= rem:
            pref.sum(Pos(j + 1 - rem)..=p)
        else:
            pref.sum(Pos.first..=p) + pref.sum(Pos(n + j + 1 - rem)..=Pos.last)
        res[p] = total * full + tail
    res

var dp: [0..=k_max, Pos]Z = 0
for i in 1..=l + k_max:
    let inv_i = comb.inv(i)
    for k in (0..=min(k_max, i)).rev:
        var next_row: [Pos]Z = 0
        let (p_lock, p_key): (Z, Z) = if i == 1 or k == i:
            (1, 0)
        else:
            let denom = comb.inv(2 * i - k - 2)
            ((i - 1) * denom, (i - k - 1) * denom)
        if k < i:
            let rotated = [Pos] p => dp[k, shift_back(p, 1)] + if p == Pos.first: Z(1) else: Z(0)
            let w_all = cyclic_window_sum(rotated, i)
            let w_real = cyclic_window_sum(rotated, i - k)
            let (c_lock, c_key) = (inv_i * p_lock, inv_i * p_key)
            for p in Pos: next_row[p] += c_lock * w_all[p] + c_key * w_real[p]
        if k > 0:
            let c_fake = k * inv_i * p_key
            let off = i - k
            for p in Pos: next_row[p] += c_fake * dp[k - 1, shift_back(p, off)]
        for p in Pos: dp[k, p] = next_row[p]

print([dp[k_max, p] for p in Pos])
