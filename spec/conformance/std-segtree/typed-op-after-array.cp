// expect: E0302
domain D = 0..3
let a: [D]Int = [1, 2, 3]
let s = SegTree<D, Int>(a, +) // <- here
