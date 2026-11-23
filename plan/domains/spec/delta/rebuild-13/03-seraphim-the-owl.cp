// Codeforces 1945D Seraphim the Owl
// in: t; per test n m, then a (n ints), b (n ints); out: per test, the minimum coins to reach one of the first m places
@!multitest
let n: 1..=2e5 = read()
let m: 1..=n = read()
domain P = 1..=n
let a = read<[P]1..=1e9>()
let b = read<[P]1..=1e9>()

var behind = 0
var best: ?Int = None
for i in P.rev:
    if Int(i) <= m:
        let cost = a[i] + behind
        if best == None or cost < best!: best = cost
    behind += min(a[i], b[i])
print(best!)
