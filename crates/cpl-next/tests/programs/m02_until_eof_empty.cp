// expect: ok, output "0"
// input: "\n \n"
// `until_eof` with nothing but blank space in the input runs no case.
print(0)
@!multitest(until_eof)
let n: Int = read()
print(n)
