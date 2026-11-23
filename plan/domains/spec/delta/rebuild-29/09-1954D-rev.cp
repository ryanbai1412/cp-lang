// CF 1954D Colored Balls, revised: sum the values of only the colour sets with an even
// number of colours. The knapsack gains a parity dimension.

type Z = Mod<998244353>
let n: 1..=5000 = read()
let a = read<[n]1..=5000>().sorted()
var ways: [2, 5001]Z = 0               // ways[q, s]: sets of earlier colours, size parity q, s balls
ways[0, 0] = 1
var total: Z = 0
var sum = 0
for x in a:
    for s in 0..=sum: total += ways[1, s] * max(x, (s + x + 1) / 2)
    for s in (0..=sum).rev, q in 0..2: ways[1 - q, s + x] += ways[q, s]
    sum += x
print(total)
