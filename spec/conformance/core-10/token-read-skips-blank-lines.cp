// expect: ok, output "9"
// rule: 10.2.8
// input: "2\n\n3\n\n\n4\n"
// A token read skips every run of whitespace, so blank lines between tests are invisible to it.
let a: Int = read()
let b: Int = read()
let c: Int = read()
print(a + b + c)
