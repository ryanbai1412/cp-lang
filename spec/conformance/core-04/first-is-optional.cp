// expect: ok, output "1"
domain V = 1..=read<1..=5>()
print(V.first!)
