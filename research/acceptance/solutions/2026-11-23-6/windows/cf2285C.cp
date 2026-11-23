type Z = Mod<1_000_000_007>

@!multitest
let n: 1..=2e5 = read()
let a: [0..n]1..=1e9 = read()
let c = a.sorted(by: desc)
var ans = Z(1)
for j in 0..n:
    let nxt = if j + 1 < n: c[j + 1] else: 0
    if c[j] <= nxt: break
    ans += Z(c[j] - 1) ** (j + 1)
print(ans)
