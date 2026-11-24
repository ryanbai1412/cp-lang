// expect: ok, output "9 9 3"
// rule: 08.1.9
// Leaving `op` out binds `new`'s op type from its default `NoOp()` before the bound `A: Acts<M.Fold, F>`, so `impl<T> Acts<T, ?T> for AssignAction` gives `F = ?Int` and `apply` takes `Some(9)`.
use std.segtree.SegTree
var s = SegTree(act: assign, a: [1, 2, 3])
s.apply(0..2, Some(9))
print(s[0], s[1], s[2])
