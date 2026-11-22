// Codeforces 2180 A — Carnival Wheel
//
// Repeatedly stepping `+b` modulo `l` from `a` visits `{(a + k * b) mod l}`.
// By Bézout's identity, `{k * b mod l}` is the subgroup of multiples of
// `g = gcd(l, b)`, so the reachable values in `0..l` are exactly the residue
// class of `a` modulo `g` (`a >= 0`), whose maximum is `l - g + a % g`.

@!multitest
let (l, a, b): (1..=5_000, 0..=5_000, 1..=5_000) = read()
let g = gcd(l, b)
print(l - g + a % g)
