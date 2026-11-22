domain I = 0..4
var st = LazySegTree<I, Int>(Action(0, (f, g) => f + g, (f, x) => f + x))
