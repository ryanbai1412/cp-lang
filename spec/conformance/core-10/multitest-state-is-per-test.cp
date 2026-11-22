// expect: ok, output "3\n1"
// rule: 10.1.1
// input: "2\n3\n1\n"
@!multitest
let n: Int = read()
var seen = 0
for _ in 0..n: seen += 1
print(seen)
