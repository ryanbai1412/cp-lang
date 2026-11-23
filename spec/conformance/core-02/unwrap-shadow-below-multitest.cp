// expect: ok, output "4"
// rule: 02.4.8
// input: "1\n3\n"
// The per-test region is a body, so `let at = at!` ends the fill phase there too (rebuild round 16 R16-SHADOW-UNWRAP).
@!multitest
let n = read<1..=5>()
var at: [0..n]?Int = None
for i in 0..n: at[i] = i * 2
let at = at!
print(at[n - 1])
