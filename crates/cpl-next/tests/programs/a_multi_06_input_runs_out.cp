// expect: R0501
// rule: R10.2.6
// input: "3\n1\n2\n"
// The third case reads past the end of input: bad input token trap, after printing 1 and 2.
@!multitest
let x: Int = read()  // <- here
print(x)
