// First draft from acceptance run 1 (cold agent, docs only), Codeforces 2285C.
type Z = Mod<1_000_000_007>

@!multitest
let n: 1..=2e5 = read()
let a: [0..n]1..=1e9 = read()
let b = a.sorted(by: x => -x)  // expect: sort_desc_by_negation
var ans: Z = 1
for k in 0..n:
    if k + 1 < n and b[k] == b[k + 1]:  // expect: missing manual-adjacent-compare
        break
    ans += Z(b[k] - 1) ** (k + 1)
print(ans)
