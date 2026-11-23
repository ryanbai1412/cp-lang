// CF 2039D Shohag Loves GCD: a_i is the (Omega(i) + 1)-th largest element of S, where
// Omega counts prime factors with multiplicity; impossible iff some Omega(i) >= m.
let lim = 100_000
let sv = Sieve.new(lim)
let omega = [0..=lim] x => if x < 2: 0 else: sum(f.1 for f in sv.factor(x))

@!multitest
let (n, m): (1..=1e5, 1..=n) = read()
let s = read<[m]1..=n>()
if (1..=n).any(i => omega[i] >= m):
    print(-1)
    return
print([s[m - 1 - omega[i]] for i in 1..=n])
