// expect: ok, output "5 5 2 0"
use std.algebra.NoOp
domain D = 0..4
fn read_all(st: SegTree<D, Int, Int, NoOp>) -> List<Int> = [st.get(D(i)) for i in 0..4]
var st = SegTree(act: add, a: [D] _ => 0)
st.apply(0..2, 5)
st.apply(2..3, 2)
print(..read_all(st))
