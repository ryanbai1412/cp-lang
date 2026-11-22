// expect: ok, output "2 YES NO"
// rule: std.bitset.Subset.set
domain V = 0..5
var s = Subset<V>.empty
s.set(V(1), true)
s.set(V(3), true)
print(s.len, yes(V(1) in s), yes(V(2) in s))
