// expect: ok, output "1\n2\n3"
// rule: R10.1.1
// input: "1 2\n3"
// until_eof with no trailing newline.
@!multitest(until_eof)
let x: Int = read()
print(x)
