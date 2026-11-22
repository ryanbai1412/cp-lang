// expect: ok, output "YES"
domain V = 0..4
let o: ?Pair<V> = None
let s: ?Pair<V> = Pair<V>.last!
print(yes(o == None and s! == Pair<V>.last!))
