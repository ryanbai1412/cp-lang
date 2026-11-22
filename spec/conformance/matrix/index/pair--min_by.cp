// expect: ok, output "YES"
domain V = 0..4
print(yes(Pair<V>.min(by: x => 0)! == Pair<V>.first!))
