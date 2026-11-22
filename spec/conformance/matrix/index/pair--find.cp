// expect: ok, output "YES"
domain V = 0..4
print(yes(Pair<V>.find(x => x == Pair<V>.last!)! == Pair<V>.last!))
