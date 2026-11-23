// Source: Codeforces 2089C2 (Key of Like, hard), https://codeforces.com/contest/2089/problem/C2
// Problem: n <= 100 members take turns trying one key on one lock, always a pair most likely to
//   match given all past tries (ties uniform), until all l <= 5000 locks are open; there are
//   l + k keys, k <= 25 of them fake. Print each member's expected number of successful matches
//   mod 1e9+7. Multitest, sum l <= 5000.
// Solution: with i unused keys, k of them fake, play settles one key or one lock at a time: a lock
//   tried key by key opens after a uniform 1..i tries, a key tried lock by lock opens after a
//   uniform 1..i - k tries or, if fake, is dropped after i - k misses (k - 1 left); the two modes
//   mix as (i - 1) : (i - k - 1). dp[k, p] = expected matches of the player p turns after the
//   mover; rows grow with i, k downward so row k - 1 is still the previous round, and each
//   uniform turn shift is a cyclic window sum over a doubled prefix array: O((l + k) k n).

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
    let total = pref.fold(0..n)
    [Pos] p => total * full + pref.fold(p.ord + n + 1 - rem..p.ord + n + 1)

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
        for p in Pos: dp[k, p] = next_row[p]

print([dp[k_max, p] for p in Pos])
