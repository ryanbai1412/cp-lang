// Codeforces 1907E Good Triples
// in: t; per test n; out: per test, the number of ordered triples (a, b, c) with a + b + c = n and digsum(a) + digsum(b) + digsum(c) = digsum(n)
@!multitest
let n: 0..=1e7 = read()
print(n.digits().map(x => (x + 1) * (x + 2) / 2).prod())
