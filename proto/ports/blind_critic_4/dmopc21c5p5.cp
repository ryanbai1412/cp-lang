// DMOPC '21 Contest 5 P5 — Prime Difference Permutation: find a permutation of
// `1..=n` (`n <= 100`) maximizing `sum_i w(p_i, p_{i+1})`, where
// `w(a, b) = |a - b|` if `|a - b|` is prime and `0` otherwise.
//
// Any permutation satisfies the unconstrained alternating bound
// `sum |p_i - p_{i+1}| <= (n * n) / 2 - 1`, which is strict for `n >= 13`
// because every odd prime flips parity (disconnecting the low/high bipartite
// halves except via the single even prime `2`).
//
// - For `n <= 12`, Held-Karp DP over `(Subset<V>, V)` finds the provably exact
//   maximum-weight Hamiltonian path in `O(n^2 2^n)` with `?Int`/`?V`.
// - For `n > 12`, this implements the reference heuristic: 64 deterministic
//   `Rng`-seeded greedy starts refined to 2-opt local maxima (no reversal
//   `perm[i + 1..=j]` strictly increases `score(perm)`), keeping the highest-
//   scoring permutation found. No global optimality guarantee is claimed for
//   `n > 12`; the output is guaranteed to be a valid 2-opt local optimum.

use std.rand.{Rng}

let n: 1..=100 = read()
domain Pos = 0..n
let sv = Sieve.new(n)

fn edge_w(a: Int, b: Int) -> Int:
    let d = abs(a - b)
    if sv.is_prime(d): d else: 0

fn score(a: [Pos]Int) -> Int = sum(edge_w(a[i], a[i.next!]) for i in Pos.first..Pos.last)

fn solve_exact(m: 1..=12) -> List<Int>:
    domain V = 0..m
    var dp: [Subset<V>, V]?Int = none
    var prv: [Subset<V>, V]?V = none
    for v in V:
        dp[Subset<V>.empty.with(v), v] = some(0)
    for mask in Subset<V>:
        for last in V if last in mask:
            if let cur = dp[mask, last]:
                for nxt in V if nxt not in mask:
                    let nmask = mask.with(nxt)
                    let cand = cur + edge_w(last.ord + 1, nxt.ord + 1)
                    if dp[nmask, nxt].map(old => cand > old) ?? true:
                        dp[nmask, nxt] = some(cand)
                        prv[nmask, nxt] = some(last)
    let full = Subset<V>.full
    let best_end = argmax([V] v => dp[full, v]!)!
    var out: List<Int> = []
    var cm = full
    var cv: ?V = some(best_end)
    while let u = cv:
        out.push(u.ord + 1)
        cv = prv[cm, u]
        cm = cm.without(u)
    out.reverse()
    out

fn two_opt_step(mut perm: [Pos]Int) -> Bool:
    for i in Pos if let i1 = i.next, let j_start = i1.next:
        for j in j_start..Pos.last:
            let j1 = j.next!
            let (a, b, c, d) = (perm[i], perm[i1], perm[j], perm[j1])
            if edge_w(a, c) + edge_w(b, d) > edge_w(a, b) + edge_w(c, d):
                perm[i1..=j].reverse()
                return true
    false

fn refine_two_opt(mut perm: [Pos]Int):
    var moving = true
    while moving:
        moving = two_opt_step(mut perm)

fn greedy_start(mut rng: Rng, trial: Int) -> [Pos]Int:
    var cur = if trial == 0: 1 else: rng.int(1, n)
    var rem = [v for v in 1..=n if v != cur]
    var perm: [Pos]Int = 0
    perm[Pos.first] = cur
    for p in Pos if p != Pos.first:
        let pick = argmax([0..rem.len] i => (edge_w(cur, rem[i]), rng.int(0, 1_000_000_000)))!
        cur = rem.remove(pick)
        perm[p] = cur
    perm

if n <= 12:
    print(solve_exact(n), sep: "\n")
else:
    var rng = Rng.new(123456789)
    var best_perm: [Pos]Int = [Pos] p => p.ord + 1
    var best_sc = score(best_perm)
    for trial in 0..64:
        var perm = greedy_start(mut rng, trial)
        refine_two_opt(mut perm)
        let sc = score(perm)
        if sc > best_sc:
            best_sc = sc
            best_perm = perm
    print(best_perm, sep: "\n")
