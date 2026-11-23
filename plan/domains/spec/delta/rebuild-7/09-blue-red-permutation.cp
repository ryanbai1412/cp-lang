// Codeforces 1607D Blue-Red Permutation
// in: t; per test n, then a_1..a_n, then a colour string of 'B'/'R'. out: per test YES if a permutation of 1..n is reachable, else NO.
@!multitest
let n: 1..=2e5 = read()
let a = read<[n]-1e9..=1e9>()
let colour = read<Str of "BR">()
let blue = [a[i] for i in 0..n if colour[i] == 'B'].sorted()
let red = [a[i] for i in 0..n if colour[i] == 'R'].sorted()
let ok = blue.items.all((i, x) => x >= i + 1) and red.items.all((j, x) => x <= blue.len + j + 1)
print(yes(ok))
