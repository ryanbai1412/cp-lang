// CF 1912K Kim's Quest. Only parities matter: dp[m] counts subsequences of
// length >= 2 whose last two parities are the bits of m = (p1 << 1) | p2.
// Appending parity c to them is allowed when p1 + p2 + c is even.
type Z = Mod<998244353>
let n: 3..=2e5 = read()
let a = read<[n]1..=2e5>()
var one: [2]Z = 0
var dp: [4]Z = 0
var ans: Z = 0
for x in a:
    let c = x % 2
    var add: [4]Z = 0
    for m in 0..4:
        let p1 = m >> 1
        let p2 = m & 1
        if (p1 + p2 + c) % 2 == 0:
            add[(p2 << 1) | c] += dp[m]
            ans += dp[m]
    for p in 0..2: add[(p << 1) | c] += one[p]
    for m in 0..4: dp[m] += add[m]
    one[c] += 1
print(ans)
