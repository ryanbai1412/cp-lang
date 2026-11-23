// expect: E0302 at 2
domain V = 0..5
let t = SegTree<V, Int>.of(max, v => "x")
