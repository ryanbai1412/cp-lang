// `AddMin` folds with the bare operator `min`: an operator holds no value,
// so it is not a parameter of the C++ the constructor becomes.
let n: 1..=10 = read()
domain Slot = 0..n
var tree = SegTree<Slot, Int, Int>(min, [Slot] i => i.ord, act: Action<Int, Int>(0, +, (f, x, _len) => x + f, over: min))
tree.apply(Slot.first!..=Slot.last!, 5)
tree.apply(Slot.first!..=Slot.first!, 9)
print(tree.fold(Slot.first!..=Slot.last!), tree.get(Slot.first!), tree.get(Slot.last!))
