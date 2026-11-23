// cf1475g Strange Beauty: a beautiful array is a divisibility chain. dp[x] = cnt[x] +
// max dp over proper divisors of x; the shared divisor lists are built once for all tests.
const LIM = 2e5
domain X = 1..=LIM
var divs: [X]List<X> = [X] _ => List<X>()
for d in X:
    var m = 2 * Int(d)
    while m <= LIM:
        divs[X(m)].push(d)
        m += Int(d)

@!multitest
let n: 1..=2e5 = read()
let a = read<[n]X>()
var cnt: [X]Int = 0
for v in a: cnt[v] += 1
var dp: [X]Int = 0
for x in X:
    let below = divs[x].map(d => dp[d]).max() ?? 0
    dp[x] = cnt[x] + below
print(n - dp.max()!)
