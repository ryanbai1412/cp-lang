// DMOPC '21 Contest 5 P5 — Prime Difference Permutation (runnable proto/cpc port)
// Prototype deltas vs v2.1 draft: no labeled `'improve:` loop (lexer crash on `'label:`),
// no `Sieve`, no `U64`, explicit sieve + constructive/2-opt search (plus exact Subset<V> DP when n <= 12).

let n: 1..=100 = read()

var is_prime: List<Bool> = []
for i in 0..=n:
    is_prime.push(i >= 2)
var p = 2
while p * p <= n:
    if is_prime[p]:
        var m = p * p
        while m <= n:
            is_prime[m] = false
            m += p
    p += 1

fn edge_w(a: Int, b: Int) -> Int:
    let d = if a > b: a - b else: b - a
    return if is_prime[d]: d else: 0

fn solve_exact(m_n: 1..=12):
    // Exact Held-Karp DP for small n (proto/cpc lacks `for s in Subset<V>:` iteration;
    // uses bitmask domain `M = 0..(1 << m_n)` as the runnable equivalent).
    let total_masks = 1 << m_n
    domain M = 0..total_masks
    domain V = 0..m_n
    var dp: [M, V]Int = -1
    var prv: [M, V]Int = -1
    for v in 0..m_n:
        dp[M.from_idx(1 << v)!, V.from_idx(v)!] = 0
    for mask in 1..total_masks:
        let mm = M.from_idx(mask)!
        for last in 0..m_n:
            let cur = dp[mm, V.from_idx(last)!]
            if cur < 0:
                continue
            for nxt in 0..m_n:
                if (mask & (1 << nxt)) != 0:
                    continue
                let nmask = M.from_idx(mask | (1 << nxt))!
                let vn = V.from_idx(nxt)!
                let cand = cur + edge_w(last + 1, nxt + 1)
                if cand > dp[nmask, vn]:
                    dp[nmask, vn] = cand
                    prv[nmask, vn] = last
    let full = total_masks - 1
    let fm = M.from_idx(full)!
    var best_end = 0
    var best_val = -1
    for v in 0..m_n:
        if dp[fm, V.from_idx(v)!] > best_val:
            best_val = dp[fm, V.from_idx(v)!]
            best_end = v
    var out: List<Int> = []
    var cm = full
    var cv = best_end
    for _ in 0..m_n:
        out.push(cv + 1)
        let pv = prv[M.from_idx(cm)!, V.from_idx(cv)!]
        cm = cm ^ (1 << cv)
        if pv >= 0:
            cv = pv
    out.rev()
    for i in 0..m_n:
        print(out[i])

if n <= 12:
    solve_exact(n)
else:
    // General n: multi-start greedy + 2-opt local search (no RNG in cheat sheet; LCG in Int)
    var rng = 123456789
    fn next_rand(lim: Int) -> Int:
        rng = (rng * 1103515245 + 12345) % 2147483647
        return rng % lim

    fn score(a: List<Int>) -> Int:
        var s = 0
        for i in 0..a.len - 1:
            s += edge_w(a[i], a[i + 1])
        return s

    var best_perm: List<Int> = []
    for i in 1..=n:
        best_perm.push(i)
    var best_sc = score(best_perm)

    for trial in 0..64:
        var perm: List<Int> = []
        var used: List<Bool> = []
        for _ in 0..=n:
            used.push(false)
        var cur = if trial == 0: 1 else: 1 + next_rand(n)
        perm.push(cur)
        used[cur] = true
        for _ in 1..n:
            var nxt = -1
            var best_d = -1
            for cand in 1..=n:
                if not used[cand]:
                    let w = edge_w(cur, cand)
                    if w > best_d or (w == best_d and next_rand(3) == 0):
                        best_d = w
                        nxt = cand
            perm.push(nxt)
            used[nxt] = true
            cur = nxt

        var improved = true
        while improved:
            improved = false
            for i in 0..n - 2:
                if improved:
                    break
                for j in i + 2..n - 1:
                    let old_w = edge_w(perm[i], perm[i + 1]) + edge_w(perm[j], perm[j + 1])
                    let new_w = edge_w(perm[i], perm[j]) + edge_w(perm[i + 1], perm[j + 1])
                    if new_w > old_w:
                        var l = i + 1
                        var r = j
                        while l < r:
                            perm.swap(l, r)
                            l += 1
                            r -= 1
                        improved = true
                        break
        let sc = score(perm)
        if sc > best_sc:
            best_sc = sc
            best_perm = perm

    for i in 0..n:
        print(best_perm[i])
