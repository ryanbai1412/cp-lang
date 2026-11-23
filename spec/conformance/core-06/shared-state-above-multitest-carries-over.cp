// expect: ok, output "1 1 1\n3 2 1\n6 3 1"
// input: "3\n1\n2\n3"
// rule: 06.5.1
// A shared `var` and a shared list above `@!multitest` keep each case's changes for the next case.
var total = 0
var log: List<Int> = []
@!multitest
let k: Int = read()
total += k
log.push(k)
print(total, log.len, log[0])
