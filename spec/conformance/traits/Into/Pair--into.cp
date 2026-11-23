// expect: ok, output "2 3"
domain V = 0..4
print(Int(Pair<V>(V(2), V(1))), Int(Pair<V>(V(0), V(3))))
