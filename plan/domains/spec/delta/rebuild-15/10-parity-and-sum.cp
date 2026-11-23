// CF 1993B Parity and Sum
// in: t; per test n, a_1..a_n. out: the fewest operations to make all parities equal.
@!multitest
let n = read<1..=2e5>()
let a = read<[n]1..=1e9>()
let evens = a.filter(v => v % 2 == 0).sorted()
if evens.len == 0 or evens.len == n:
    print(0)
    return
var big = a.filter(v => v % 2 == 1).max()!
var ops = evens.len
for e in evens:
    if big < e:
        ops += 1
        break
    big += e
print(ops)
