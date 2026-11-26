// expect: ok, output "20000100000"
// rule: R07.4.6
// Each slice points at `xs`: 200001 slices of a 200000-element list stay fast.
let n = 200000
var xs: List<Int> = []
for i in 0..n: xs.push(i)
var total = 0
for i in 0..=n: total += xs[i..n].len
print(total)
