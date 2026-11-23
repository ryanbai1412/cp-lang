// expect: ok, output "5\n3"
// rule: 10.1.1
// input: "2\n3\n1\n"
// A per-test domain may size itself from another domain's `.count` (rebuild round 18 H04).
@!multitest
let n = read<1..=5>()
domain C = 0..2
domain N = 0..n + C.count
print(N.count)
