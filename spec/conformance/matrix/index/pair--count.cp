// expect: ok, output "1"
domain V = 0..4
print(Pair<V>.count(x => x == Pair<V>.last!))
