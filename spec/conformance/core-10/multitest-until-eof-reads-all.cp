// expect: ok, output "4\n9"
// rule: 10.1.1
// input: "2 2\n3 3\n"
@!multitest(until_eof)
let a: Int = read()
let b: Int = read()
print(a * b)
