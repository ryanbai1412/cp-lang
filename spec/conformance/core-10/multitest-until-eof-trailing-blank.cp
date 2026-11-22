// expect: ok, output "1\n2"
// rule: 10.1.1
// input: "1\n2\n\n  \n"
@!multitest(until_eof)
let n: Int = read()
print(n)
