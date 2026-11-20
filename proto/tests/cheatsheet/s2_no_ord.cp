type Item = (id: Int, d: ?Int)
var xs: List<Item> = [(id: 1, d: none), (id: 2, d: 5), (id: 3, d: 9)]
sort(mut xs, by: x => (x.d == none, -(x.d ?? 0)))
print(xs[1].id, xs[1].d ?? 0, xs[2].id)
