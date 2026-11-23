// CF 1850F We Were Both Children
// in: t; n, a_1..a_n jump lengths | out: per test the most frogs one trap in 1..=n catches
@!multitest
let n: 1..=2e5 = read()
let a = read<[n]1..=1e9>()

var frogs: [1..=n]Int = 0
for x in a:
    if x <= n: frogs[x] += 1
var caught: [1..=n]Int = 0
for d in 1..=n:
    if frogs[d] > 0:
        for spot in d..=n by d:
            caught[spot] += frogs[d]
print(caught.max()!)
