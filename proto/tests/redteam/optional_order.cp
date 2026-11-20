// `?T` has an order where `T` has one, with none before every value, so records holding one sort and go in a heap.
var xs: List<(Int, ?Int)> = [(2, 5), (1, none), (1, 3), (2, none)]
let ys = sorted(xs)
for (a, b) in ys: print(a, b ?? -1)
var h = MinHeap<(Int, ?Int)>()
h.push((4, none))
h.push((4, 1))
h.push((3, 9))
let (p, q) = h.pop()!
print(p, q ?? -1)
var zs: List<?Int> = [3, none, 1]
print([z ?? 0 for z in sorted(zs)])
