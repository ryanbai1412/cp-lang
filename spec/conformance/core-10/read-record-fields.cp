// expect: ok, output "7"
// rule: 10.2.1
// input: "3 4\n"
let r = read<(a: Int, b: Int)>()
print(r.a + r.b)
