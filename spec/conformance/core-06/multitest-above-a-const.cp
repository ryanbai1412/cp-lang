// expect: ok, output "7\n8"
// input: "2\n4\n5\n"
// rule: 06.5.1
// `@!multitest` divides the file even when the item right after it is a hoisted `const`.
@!multitest
const K = 3
let n: 0..=9 = read()
print(n + K)
