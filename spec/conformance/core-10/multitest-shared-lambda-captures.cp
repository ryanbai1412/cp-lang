// expect: ok, output "6\n10"
// rule: 10.1.1
// input: "2\n4\n5\n"
// Each test may call a lambda from the shared section that captures a shared `Comb` (rebuild round 19 H05).
type Z = Mod<1_000_000_007>
let c = Comb<Z>()
let pairs = (n: Int) => c.choose(n, 2)
@!multitest
let n = read<Int>()
print(pairs(n))
