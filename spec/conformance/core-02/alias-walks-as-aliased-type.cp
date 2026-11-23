// expect: ok, output "0"
// A type alias stands for its type everywhere, so `for g in G` walks `Subset<R>` (bug-catch round 14, 28ca9a32).
domain R = 0..3
type G = Subset<R>
var s: [G]Int = 0
for g in G: s[g] = g.len
print(s[G.empty])
