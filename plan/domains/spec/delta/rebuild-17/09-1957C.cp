// 1957C How Does the Rook Move?: f(i) = f(i-1) + 2(i-1) f(i-2) over the free rows.
type Z = Mod<1e9 + 7>
const N = 300000
var f: [N + 1]Z = 0
f[0] = 1
f[1] = 1
for i in 2..=N:
    f[i] = f[i - 1] + 2 * (i - 1) * f[i - 2]

@!multitest
let n: 1..=3e5 = read()
let k: 0..=n = read()
var free = n
repeat k:
    let r: 1..=n = read()
    let c: 1..=n = read()
    free -= if r == c: 1 else: 2
print(f[free])
