// Codeforces 1899D: Yarik and Musical Notes
// I/O: t tests; each n then a_1..a_n (note b_i = 2^a_i); print the number of pairs i < j with b_i^b_j = b_j^b_i.
@!multitest
let n: 1..=2e5 = read()
let a = read<[n]1..=1e9>()
// equal exponents always commute; the only unequal pair is {1, 2} (2^4 = 4^2)
let c = a.tally()
print(c.values.sum(of: k => k * (k - 1) / 2) + c[1] * c[2])
