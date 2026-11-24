// expect: E0302
domain V = 0..5
let t = SegTree<V, Int>.of(max, v => "x") // <- here
