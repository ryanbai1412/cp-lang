// Codeforces 1921D Very Different Array
// in: t; per test n m, then a (n ints), b (m ints); out: per test, the maximum total difference D
@!multitest
let n: 1..=2e5 = read()
let m: n..=2e5 = read()
var a = read<[n]1..=1e9>()
var b = read<[m]1..=1e9>()
a.sort()
b.sort()

// The k smallest a take the k largest b; the rest take the smallest b, mirrored.
fn high(i: Int) -> Int = abs(a[i] - b[m - 1 - i])
fn low(i: Int) -> Int = abs(a[i] - b[n - 1 - i])

var total = a.keys.map(low).sum()
var best = total
for i in a.keys:
    total += high(i) - low(i)
    best max= total
print(best)
