// cf1312d Count the Arrays: pick n - 1 distinct values, the largest is the peak, one of
// the other n - 2 repeats, and each of the remaining n - 3 goes left or right.
type Z = Mod<998244353>
let n: 2..=2e5 = read()
let m: n..=2e5 = read()
let comb = Comb<Z>(m)
let ans: Z = if n == 2: 0 else: comb.choose(m, n - 1) * (n - 2) * Z(2).pow(n - 3)
print(ans)
