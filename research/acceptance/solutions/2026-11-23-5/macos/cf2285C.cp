type Z = Mod<1_000_000_007>
@!multitest
let n: 1..=2e5 = read()
let a: [0..n]1..=1e9 = read()
let b = a.sorted(by: desc)
var ans: Z = 1
for t in 0..n:
    if t + 1 < n and b[t] == b[t + 1]: break
    ans += Z(b[t] - 1) ** (t + 1)
print(ans)
