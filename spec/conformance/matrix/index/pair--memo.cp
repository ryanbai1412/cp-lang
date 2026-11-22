// expect: ok, output "7"
domain V = 0..4
@memo(map) fn f(x: Pair<V>) -> Int = 7
print(f(Pair<V>.last!))
