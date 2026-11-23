// expect: ok, output "1"
// rule: 04.2.3 04.8.4
// input: "3\n"
domain V = 1..=read<1..=5>()
print(V.first!)
