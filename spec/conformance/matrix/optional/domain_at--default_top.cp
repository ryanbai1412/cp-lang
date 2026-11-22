// expect: ok, output "3"
domain V = 0..5
print(V.at(3) ?? V(0))
