// expect: ok, output "1\n1"
// rule: R10.1.1, R05.10.1
// input: "2\n7\n7\n"
// A closure made in a case shares that case's var only.
@!multitest
let x: Int = read()
var cnt = 0
let inc = () => cnt += 1
inc()
print(cnt)
