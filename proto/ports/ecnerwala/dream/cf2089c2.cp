// Codeforces 2089 C2: Key of Like (Hard Version).
// `n` players take turns picking uniformly from `i` remaining locks (`k`
// counterfeit, `i - k` real), as `i` grows from 1 to `l + k_max`.
// `dp[k, p]` is the expected number of real locks opened by the player `p`
// steps after the current turn.
//
// For `i > 1` and `k < i`, the next lock/key draw splits into `(i - 1)`
// lock-first outcomes (`p_lock`) and `(i - k - 1)` key-first outcomes
// (`p_key`), out of `(i - 1) + (i - k - 1) = 2i - k - 2` equally likely
// outcomes:
//   - Opening a real lock (`k < i`) scores +1 for `Pos.first` and shifts the
//     turn uniformly by `1..=i` (weight `p_lock / i`) or `1..=i - k` (weight
//     `p_key / i`), summed via `cyclic_window_sum`.
//   - Opening a counterfeit lock (`k > 0`) transitions `k -> k - 1` with
//     weight `k * p_key / i` and rotates by `i - k`.
// Iterating `k` downward lets row `k` read row `k - 1` from round `i - 1` in
// place without a second table.

type Z = Mod<1_000_000_007>
let comb = Comb<Z>()

@!multitest
let n: 1..=100 = read()
let l: 1..=5_000 = read()
let k_max: 0..=100 = read()
domain Pos = 0..n

fn shift_back(p: Pos, off: Int) -> Pos = Pos((p.ord + n - off % n) % n)

// For each p in Pos, sums `ways[p - t mod n]` over t in 0..len.
// Doubling `ways` turns the wrapped remainder window into one contiguous slice.
fn cyclic_window_sum(ways: [Pos]Z, len: Int) -> [Pos]Z:
    let doubled = List(ways) + List(ways)
    let pref = prefix(doubled)
    let (full, rem) = (len / n, len % n)
    let total = pref.sum(0..n)
    [Pos] p => total * full + pref.sum(p.ord + n + 1 - rem..p.ord + n + 1)

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
            let rotated = [Pos] p => dp[k, shift_back(p, 1)] + Z(Int(p == Pos.first))
            let w_all = cyclic_window_sum(rotated, i)
            let w_real = cyclic_window_sum(rotated, i - k)
            let (c_lock, c_key) = (inv_i * p_lock, inv_i * p_key)
            for p in Pos: next_row[p] += c_lock * w_all[p] + c_key * w_real[p]
        if k > 0:
            let c_fake = k * inv_i * p_key
            let off = i - k
            for p in Pos: next_row[p] += c_fake * dp[k - 1, shift_back(p, off)]
        dp[k] = next_row

print([dp[k_max, p] for p in Pos])
