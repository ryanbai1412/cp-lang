// The lowering gaps the Batch 52 ports found: `Range.step` and
// `Range.clamp`, an `Int` reduced into a `Mod<P>`, `[Bool]T`, an array
// filled from a list, an array of structs built by one call, and a
// domain's positions sorted by a key. `Mod`, `Fenwick` and `SortedBag`
// are used without a `use`.
const P = 7
type Z = Mod<P>
struct Pair:
    a: Z; b: Z

for i in (0..=10).step(3): print(i)
print((2..9).step(2))

domain Grid = 1..=8
var bit: Fenwick<Grid, Int> = Fenwick()
for g in Grid: bit.add(g, g.label)
for (l, r) in [(-3, 4), (2, 5), (7, 20), (9, 12), (-5, 0), (1, 8), (5, 2)]:
    let cut = (l..=r).clamp(Grid)
    print(cut.len, bit.sum(cut))

var total: Z = 5
total += 4
let pair = Pair(10, -1)
print(total, pair.a, pair.b)

var dp: [Bool]Z = [1, 0]
dp[true] += 12
dp[total == Z(2)] = dp[false] + dp[true]
print(dp[false], dp[true])
var primes: [4]Int = [2, 3, 5, 7]
print(sum(primes[i] for i in 0..4))

var bags: [0..2, 0..3]SortedBag<Int> = SortedBag()
bags[1, 2].add(4)
bags[1, 2].add(1)
bags[0, 0].add(9)
print(bags[1, 2].len, bags[0, 0].len, bags[1, 1].len)

let weight: [Grid]Int = [Grid] g => (g.label * 5) % 8
print([g.label for g in sorted(Grid, by: g => weight[g])])
