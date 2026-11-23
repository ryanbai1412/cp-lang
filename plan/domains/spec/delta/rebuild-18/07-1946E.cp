// CF1946E Girl Permutation. n sits at p[m1] == s[1]; each side splits
// recursively at the next maximum: choose which values go left of it, and
// arrange the cells strictly between two maxima freely.
type Z = Mod<1e9 + 7>
let comb = Comb<Z>(2e5)

@!multitest
let n: 1..=2e5 = read()
let m1: 1..=n = read()
let m2: 1..=n = read()
let p = read<[m1]1..=n>()
let s = read<[m2]1..=n>()
if p[0] != 1 or s[m2 - 1] != n or p[m1 - 1] != s[0]:
    print(0)
    return
var ans = comb.choose(n - 1, s[0] - 1)
for (x, y) in p.adjacent:
    ans *= comb.choose(y - 2, x - 1) * comb.fact(y - x - 1)
for (x, y) in s.adjacent:
    ans *= comb.choose(n - x - 1, n - y) * comb.fact(y - x - 1)
print(ans)
