// CF 1800D Remove Two Letters
// in: t; per test n, then s (lowercase, |s| = n). out: number of distinct strings after deleting two adjacent letters.
@!multitest
let n = read<3..=2e5>()
let s = read<Str>()
// deleting at i and at i + 1 give the same string exactly when s[i] == s[i + 2]
print(n - 1 - zip(s, s.drop(2)).count((x, y) => x == y))
