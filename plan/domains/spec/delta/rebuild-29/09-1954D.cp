// CF 1954D Colored Balls: a set of colours needs max(biggest, ceil(total / 2)) groups.
// Take colours in increasing count; when x is the biggest so far, ways[s] counts the sets of
// earlier colours with s balls (a 0/1 knapsack mod 998244353).

type Z = Mod<998244353>
let n: 1..=5000 = read()
let a = read<[n]1..=5000>().sorted()
var ways: [5001]Z = 0
ways[0] = 1
var total: Z = 0
var sum = 0
for x in a:
    for s in 0..=sum: total += ways[s] * max(x, (s + x + 1) / 2)
    for s in (0..=sum).rev: ways[s + x] += ways[s]
    sum += x
print(total)
