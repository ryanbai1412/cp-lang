// Source: CCO 2003 P4 (Constrained Permutations), https://dmoj.ca/problem/cco03p4
// limits: 1000 ms, 16 MB
// Problem: n <= 9 and k constraints "x comes before y". Print how many permutations of 1..=n
//   satisfy them all.
// Solution: place the numbers front to back; need[y] is the mask of numbers that must come
//   before y. dp[s] counts orders of the set s placed first; y can join s when need[y] is in s.
//   O(2^n n).
// Runnable today on proto/cpc; matches the v2.1 Subset<V> solution.
// original: corpus/submissions/5024536.cpp

fn bit(k: Int) -> Int: return 1 << k

fn main_solve():
    let n: Int = read()
    let k: Int = read()
    domain V = 0..n
    var need: [V]Int = 0
    for _ in 0..k:
        let x: Int = read()
        let y: Int = read()
        if x != y: need[V(y - 1)] |= bit(x - 1)

    let total = bit(n)
    domain Mask = 0..total
    var dp: [Mask]Int = 0
    dp[Mask(0)] = 1
    for s in 0..total:
        let ways = dp[Mask(s)]
        if ways == 0: continue
        for v in 0..n:
            let b = bit(v)
            if (s & b) == 0 and (need[V(v)] & s) == need[V(v)]:
                dp[Mask(s | b)] += ways
    print(dp[Mask(total - 1)])

main_solve()
