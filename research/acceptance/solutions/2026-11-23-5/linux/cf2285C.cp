use std.mod.{Mod}
type Z = Mod<1_000_000_007>

@!multitest
let n: 1..=2e5 = read()
let a: [0..n]1..=1e9 = read()
let b = a.sorted(by: desc)
var ans: Z = 1
for k in 0..n:
    if k + 1 < n and b[k] == b[k + 1]: break
    ans += Z(b[k] - 1) ** (k + 1)
print(ans)
