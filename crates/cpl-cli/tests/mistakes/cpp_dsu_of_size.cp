// here: 5
// C++'s `DSU d(n)` habit: `Dsu<V>(n)` read `n` as `rollback: Bool`, no fix.
use std.dsu.{Dsu}
domain V = 0..5
var d = Dsu<V>(5)
print(1)
