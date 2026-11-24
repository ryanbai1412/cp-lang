// expect: E0302
// tool: check
// has: "help: write `SegTree<D, Int, Int, NoOp>` in full: a type param left out takes its default"
domain D = 0..4
fn first(st: SegTree<D, Int, Int>) -> Int = st.get(D(0))
let st = SegTree<D, Int, Int>(act: add, a: [D] _ => 0)
print(first(st)) // <- here
