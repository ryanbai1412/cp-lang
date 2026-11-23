// 2039D Shohag Loves GCD: a_i is the (Omega(i) + 1)-th largest of S, where
// Omega counts prime factors with multiplicity; -1 when S is too small.
let top = 1e5
let sv = Sieve(top)
var omega = List.filled(top + 1, 0)
for x in 2..=top: omega[x] = omega[x / sv.spf[x]] + 1

@!multitest
let (n, m): (1..=1e5, 1..=n) = read()
let s = read<[m]1..=n>()
if (1..=n).any(i => omega[i] >= m):
    print(-1)
    return
print([s[m - 1 - omega[i]] for i in 1..=n])
