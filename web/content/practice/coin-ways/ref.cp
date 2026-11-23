type Z = Mod<1_000_000_007>
let (n, x): (1..=100, 1..=1e6) = read()
let c: [0..n]1..=1e6 = read()
var ways: [0..=x]Z = Z(0)
ways[0] = Z(1)
for v in c:
    for s in v..=x:
        ways[s] += ways[s - v]
print(ways[x])
