// expect: ok, output "2 2"
// rule: std.bitset.Bitset.each
// A bitset's folds take its positions, as its `each` yields them: `by:` sees an `S`, not an `Int`.
domain S = 1..=5
var b = Bitset<S>.new()
b.add(S(2))
let s: S = b.max(by: s => s.label)!
print(s, max(b, by: s => s.label)!)
