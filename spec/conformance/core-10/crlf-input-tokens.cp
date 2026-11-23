// expect: ok, output "5 ab"
// rule: 10.2.8
// input: "4\r\nab\r\n"
let n = read<Int>()
let s = read<Str>()
print(n + 1, s)
