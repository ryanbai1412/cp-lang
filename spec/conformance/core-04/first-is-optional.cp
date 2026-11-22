// expect: ok, output "1"
// input: "3\n"
domain V = 1..=read<1..=5>()
print(V.first!)
