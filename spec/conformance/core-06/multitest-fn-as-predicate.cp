// expect: ok, output "4\n2"
// rule: 06.1.7
// input: "2\n3\n1\n"
// A fn below `@!multitest` captures that case's names and passes as a value (rebuild round 17 H12).
@!multitest
let n = read<1..=9>()
fn big(x: Int) -> Bool = x > n
print((1..=5).find(big) ?? 0)
