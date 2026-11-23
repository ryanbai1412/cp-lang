// CF 1866H Happy Sets
// Nonempty sets in a valid order have strictly increasing maxima, so they
// are distinct and their order is forced; empty sets go first. With L
// nonempty sets, shift set i up by L - i: the chain is nested, and each
// value x picks the level it enters (1 + min(L, x) ways); subtract the
// chains whose first set is empty. f(L) = L! ((L+1)^(K-L+1) - L^(K-L+1)),
// and the arrays are arrange(N, L) placements of those L sets.
type Z = Mod<998244353>
let (n, k): (1..=2e5, 1..=2e5) = read()
let comb = Comb<Z>(n + k)
var ans: Z = 1
for l in 1..=min(n, k):
    let e = k - l + 1
    ans += comb.arrange(n, l) * comb.fact(l) * (Z(l + 1) ** e - Z(l) ** e)
print(ans)
