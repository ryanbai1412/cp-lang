// Codeforces 1984C1 Magnitude (Easy Version): add each a_i to c, optionally taking |c|; maximize c.
// In: t; per test n, then a_1..a_n. Out: the maximum final c per test.
@!multitest
let n = read<2..=2e5>()
let a = read<[n]-1e9..=1e9>()
// hi: the best c so far; lo: the least, which is the plain prefix sum.
var (hi, lo) = (0, 0)
for x in a:
    (hi, lo) = (max(hi + x, abs(lo + x)), lo + x)
print(hi)
