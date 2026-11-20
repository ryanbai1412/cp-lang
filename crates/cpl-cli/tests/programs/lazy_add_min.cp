// `AddMin` folds with the bare operator `min`: an operator holds no value,
// so it is not a parameter of the C++ the constructor becomes.
use std.lazy.{LazySegTree, AddMin}
let n: 1..=10 = read()
domain Slot = 0..n
var tree = LazySegTree<Slot, Int>(AddMin, [Slot] i => i.ord)
tree.apply(Slot.first..=Slot.last, 5)
tree.apply(Slot.first..=Slot.first, 9)
print(tree.fold(Slot.first..=Slot.last), tree.get(Slot.first), tree.get(Slot.last))
