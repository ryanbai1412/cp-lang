// Ideal CP Lang program for cf2222h (H. Counting Sort?).
// Not gated: upstream ecnerwala/contest-programming CF/2222/H/H.cpp is a
// 7-line empty template with no solution.

@!multitest
let (n, k): (1..=50, 1..=1000) = read()
let r: [1..=n]Int = read()
type M = Mod<998_244_353>

// For any array b of length n with 0 <= b_i <= n, f(b) is the frequency vector
// of 1..=n in b, so sum(f(b)) <= n. After one step, f(f(b)) depends only on the
// sorted multiset of positive entries of f(b) — i.e., an integer partition of <= n.
// Step 1: explore the functional graph of f on partitions of <= n to get g(part)
//         for every reachable partition.
// Step 2: DP over values v = 1..=n (using the upper-bound counts #{i : r_i >= v})
//         to count arrays a with 0 <= a_i <= r_i whose frequency multiset equals
//         each partition (with multinomial weighting).

fn step_freq(b: List<Int>) -> List<Int>:
    var c: [1..=n]Int = 0
    for x in b:
        if x >= 1: c[x] += 1
    return [c[i] for i in 1..=n]

fn orbit_size(start: List<Int>) -> Int:
    var seen = Map<List<Int>, Int>()
    var cur = start
    while !seen.contains(cur):
        seen[cur] = seen.len
        cur = step_freq(cur)
    return seen.len

// Precompute factorials and inverse factorials up to n.
var fact: [0..=n]M = M(1)
for i in 1..=n: fact[i] = fact[i - 1] * M(i)
var inv_fact: [0..=n]M = M(1)
inv_fact[n] = fact[n].inv()
for i in (0..n).rev(): inv_fact[i] = inv_fact[i + 1] * M(i + 1)

// Count how many positions can take value >= v.
var cap: [1..=n]Int = 0
for i in 1..=n:
    for v in 1..=r[i]:
        cap[v] += 1

// Enumerate partitions of <= n (the multiset of positive frequencies in f(a))
// and aggregate ways per orbit size p in 1..=k.
var ans: [1..=k]M = M(0)
// ... partition DP over (remaining_slots, max_part) -> orbit_size(freq) ...
print(ans)
