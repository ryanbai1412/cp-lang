// Codeforces 2150 C — Limited Edition Shop
//
// Process items in Alice's order `a[0..n]`. Maintaining the non-decreasing DP
// `dp[j]` = max profit after prefix of `a` with Bob's frontier at `j` in `b`,
// we store its positive increments in `deltas: Map<Int, Int>` (with base value
// `tot = dp[0]`).
//
// When item `cur = a[z]` (value `v = vals[cur]`, Bob-position `pos = inv_b[cur]`)
// arrives: Alice taking `cur` shifts `dp[0..pos]` by `+v` (`tot += v` and
// `deltas[pos] -= v`), then prefix-maxing restores non-negativity of `deltas`
// by cancelling any negative `deltas[pos]` against the smallest later positive
// entries `deltas.first_gt(pos)`. The answer is `dp[n] = tot + sum(deltas)`.

@!multitest
let n: 1..=200_000 = read()
domain I = 0..n
domain V = 1..=n
let vals: [V]Int = read()
let a: [I]V = read()
let b: [I]V = read()
let inv_b = b.inverse()

var tot = 0
var deltas: Map<Int, Int> = Map()
for z in I:
    let cur = a[z]
    let v = vals[cur]
    let pos = inv_b[cur].to_idx
    tot += v
    deltas[pos] -= v
    while let cur_val = deltas.get(pos) and cur_val < 0:
        if let (nxt_k, nxt_v) = deltas.first_gt(pos):
            let d = min(-cur_val, nxt_v)
            if nxt_v == d: deltas.remove(nxt_k) else: deltas[nxt_k] -= d
            if cur_val + d == 0: deltas.remove(pos) else: deltas[pos] += d
        else:
            deltas.remove(pos)

print(tot + sum(v for (_, v) in deltas))
