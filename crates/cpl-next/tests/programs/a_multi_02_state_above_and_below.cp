// expect: ok, output "1\n2"
// rule: R10.1.1
// input: "2\n5\n6\n"
// R10.1.1: state below the divider is per test; a var above persists.
var total = 0
@!multitest
var seen = 0
let x: Int = read()
seen += 1
total += 1
print(seen * 0 + total)
