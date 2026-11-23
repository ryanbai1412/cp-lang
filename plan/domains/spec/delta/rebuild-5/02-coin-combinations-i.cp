// CSES 1635 Coin Combinations I: ordered ways to make sum x from n distinct coins, mod 1e9+7.
// Input: n x, then n coin values. Output: the number of ways.
type Z = Mod<1e9 + 7>
let (n, x) = read<(1..=100, 1..=1e6)>()
let coins = read<[n]1..=1e6>()

var ways: [0..=x]Z = 0
ways[0] = 1
for s in 1..=x:
    for c in coins if c <= s:
        ways[s] += ways[s - c]
print(ways[x])
