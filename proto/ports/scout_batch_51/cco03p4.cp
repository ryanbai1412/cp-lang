// CCO '03 P4 (Constrained Permutations): count permutations of 1..=n respecting
// ordering constraints (x before y). Rethought from the original's O(n! * k)
// next_permutation scan into an idiomatic O(n * 2^n) Subset<V> bitmask DP.
// original: corpus/submissions/5024536.cpp

let (n, k): (1..=9, 0..=81) = read()
domain V = 1..=n
var need: [V]Subset<V> = Subset<V>.empty
for _ in 0..k:
    let (x, y): (V, V) = read()
    if x != y: need[y].add(x)

var dp: [Subset<V>]Int = 0
dp[Subset<V>.empty] = 1
for s in Subset<V>:
    if dp[s] == 0: continue
    for v in V if v not in s and (need[v] & s) == need[v]:
        dp[s.with(v)] += dp[s]
print(dp[Subset<V>.full])
