// C. Max Substitution
// https://codeforces.com/contest/2285/problem/C
type Z = Mod<1_000_000_007>
@!multitest
let n: 1..=2e5 = read()
let a: [0..n]1..=1e9 = read()
let b = a.sorted(by: desc)
var ans: Z = 1
for (r, run) in b.runs().items:
    if run.len > 1: break
    let v = run[run.keys.first!]
    ans += Z(v - 1) ** (r + 1)
print(ans)
