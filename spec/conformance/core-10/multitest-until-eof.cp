// expect: ok, output "start"
// rule: 10.1.1
print("start")
@!multitest(until_eof)
let n: Int = read()
print(n)
