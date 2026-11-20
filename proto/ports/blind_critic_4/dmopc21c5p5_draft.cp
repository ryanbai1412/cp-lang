// Blind Critic #4 — First Draft (written from docs/one-page-cheat-sheet.md v2.1 ONLY)
// Problem: dmopc21c5p5 (Prime Difference Permutation) — Messy Ad-Hoc / Heuristic / Non-Typeable Stress #3
// Original: corpus/submissions/4223999.cpp
// Exercises: Sieve, Marks<V>, Subset<V> exact Held-Karp DP (@brute), labeled continue 'improve (no goto!),
//            reverse(mut path[l..=r]), @checker, AND exposes missing RNG / shuffle / timer in v2.1!

let n: 1..=1000 = read()

if n <= 2:
    print([i for i in 1..=n])
    return
if n == 3:
    print([2, 1, 3])
    return

domain V = 0..n
let sv = Sieve(n)

fn edge_w(u: V, v: V) -> Int:
    let d = abs(u - v)
    if sv.is_prime[d]: d else: 0

fn score(path: List<V>) -> Int:
    // Notice: xs.adjacent in cheat sheet §10!
    sum(edge_w(a, b) for (a, b) in path.adjacent)

fn two_opt(mut path: List<V>):
    // Labeled `continue 'improve` cleanly replaces C++ `goto loop;`!
    'improve: while true:
        for i in 0..n - 2:
            for j in i + 2..n - 1:
                let a = path[i]
                let b = path[i + 1]
                let c = path[j]
                let d = path[j + 1]
                if edge_w(a, c) + edge_w(b, d) > edge_w(a, b) + edge_w(c, d):
                    reverse(mut path[i + 1..=j])
                    continue 'improve
        break

// Generation-stamped Marks<V> for O(1) .clear() across greedy restarts
var vis = Marks<V>()

fn greedy(start: V) -> List<V>:
    vis.clear()
    var path: List<V> = [start]
    vis.add(start)
    while path.len < n:
        let u = path.last!
        // BLIND-GUESS(H26b): is `v not in vis` valid, or only `not (v in vis)`?
        let nxt = max_by((v for v in V if not (v in vis)), v => edge_w(u, v))!
        path.push(nxt)
        vis.add(nxt)
    return path

// BLIND-GUESS(H25): NO RNG, NO shuffle(), NO clock()/timer anywhere in cheat sheet §1-§12!
// Forced to hand-roll xorshift64 with wrapping U64 + Fisher-Yates shuffle + fixed iteration budget.
var rng_state: U64 = 1412_88172645463325252

fn next_u64() -> U64:
    rng_state ^= rng_state << 13
    rng_state ^= rng_state >> 7
    rng_state ^= rng_state << 17
    return rng_state

fn rand_idx(lim: Int) -> Int:
    // BLIND-GUESS(H23): converting U64 -> Int after modulo
    Int(next_u64() % U64(lim))

fn shuffle_path(mut path: List<V>):
    for i in (1..path.len).rev:
        let j = rand_idx(i + 1)
        path.swap(i, j)

fn place_at(mut path: List<V>, pos: Int, target: V):
    if let idx = path.find(target):
        path.swap(pos, idx)

var best_path = greedy(V.from_idx(n / 2 - 1)!)
two_opt(mut best_path)
var best_score = score(best_path)

var cand = [v for v in V]
let restarts = if n <= 50: 40 elif n <= 100: 120 else: 200
for _ in 0..restarts:
    shuffle_path(mut cand)
    if n > 20:
        place_at(mut cand, n - 1, V.from_idx(n / 2 - 1)!)
    if n > 100:
        place_at(mut cand, 0, V.from_idx(n / 2 + 1 + rand_idx(2))!)
    var cur = cand.copy()
    two_opt(mut cur)
    let sc = score(cur)
    if sc > best_score:
        best_score = sc
        best_path = cur

print([v.to_idx + 1 for v in best_path])

// Exact Held-Karp DP for small n <= 16 exercising `Subset<V>` Index + Tropical Optionals + @memo!
// BLIND-GUESS(H26): can you iterate elements of a Subset<V> (`for u in s:`)? Cheat sheet §9 never says!
fn exact_opt_small() -> Int:
    assume n <= 16
    var dp: [Subset<V>, V]?Int = none
    for v in V:
        dp[Subset<V>.empty.with(v), v] = 0
    for mask in Subset<V>:
        for u in V if u in mask:
            if let cur = dp[mask, u]:
                for v in V if not (v in mask):
                    dp[mask.with(v), v] max= cur + edge_w(u, v)
    return max(dp[Subset<V>.full, v] for v in V)!

@checker
fn check():
    let out: [V]1..=n = read_output()
    var seen = Marks<1..=n>()
    for v in V:
        assert seen.add(out[v])
    if n <= 16:
        let got = sum(edge_w(V.from_idx(a - 1)!, V.from_idx(b - 1)!) for (a, b) in out.adjacent)
        assert got == exact_opt_small()
