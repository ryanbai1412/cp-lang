// Codeforces 1983D Swap Dilemma: equal-length swaps in a and b; can they become equal?
// In: t; per test n, then a_1..a_n, then b_1..b_n (each distinct). Out: YES/NO per test.
@!multitest
let n = read<1..=1e5>()
let a = read<[n]1..=2e5>()
let b = read<[n]1..=2e5>()
// Same values, and each swap flips both arrays' inversion parity together.
print(yes(a.sorted() == b.sorted() and a.inversions() % 2 == b.inversions() % 2))
