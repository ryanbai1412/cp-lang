// p18 with factorial tables on Mod (row 14).
type Z = Mod<1_000_000_007>
const N = 1_000_000
var fact: [0..=N]Z = Z.new(1)
for i in 1..=N: fact[i] = fact[i - 1] * Z.new(i)
var inv_fact: [0..=N]Z = Z.new(1)
inv_fact[N] = fact[N].inv()
for i in (1..=N).rev: inv_fact[i - 1] = inv_fact[i] * Z.new(i)
let n = read<1..=1e5>()
for _ in 0..n:
    let (a, b) = read<(1..=1e6, 0..=1e6)>()
    print(if b > a: Z.new(0) else: fact[a] * inv_fact[b] * inv_fact[a - b])
