// CSES 1633 Dice Combinations: ways to write n as an ordered sum of throws 1..=6.
// In: n. Out: the number of ways modulo 1e9+7.
type Z = Mod<1_000_000_007>
let n = read<1..=1e6>()
var ways: [0..=n]Z = Z(0)
ways[0] = Z(1)
for i in 1..=n:
    ways[i] = (1..=min(6, i)).map(d => ways[i - d]).sum()
print(ways[n])
