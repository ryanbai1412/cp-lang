// CF 2025E Card Game: suit 1 is a ballot sequence ending j above zero (its j spare cards
// beat the other suits' extras); every other suit is a reversed ballot sequence with k_s
// extras, and the extras must total j.
type Z = Mod<998244353>

let (n, m): (1..=500, 2..=500) = read()
let c = Comb<Z>.new(m + 1)
let w = [0..=m] j => if (m + j) % 2 == 0: c.choose(m, (m + j) / 2) - c.choose(m, (m + j) / 2 + 1) else: Z(0)
var dp: [0..=m]Z = Z(0)
dp[0] = Z(1)
repeat n - 1:
    var nd: [0..=m]Z = Z(0)
    for j in 0..=m:
        for k in 0..=(m - j):
            nd[j + k] += dp[j] * w[k]
    dp = nd
print(sum(w[j] * dp[j] for j in 0..=m))
