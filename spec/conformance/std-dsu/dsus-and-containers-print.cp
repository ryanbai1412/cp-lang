// expect: ok, output "0 0 2 0\n0 0 2 0\n0:0 0:5 2:0 0:8\n0:0 0:5 2:0 0:8\n1 2 3\n4 5\n1 3\n0 2"
// The union-finds print each vertex's root (and its offset, for PotentialDsu) in vertex order,
// without compressing paths; Deque, LinkedList, DynBitset and Subset print their elements.
use std.deque.{Deque, LinkedList}
use std.bitset.{DynBitset}
domain V = 0..4
var d = Dsu<V>()
let _ = d.unite(V(0), V(1))
let _ = d.unite(V(1), V(3))
var o = DsuOf<V, Int>(init: v => v.ord, join: (x, y) => x + y)
let _ = o.unite(V(0), V(1))
let _ = o.unite(V(1), V(3))
print(d)
print(o)
var p = PotentialDsu<V>()
let _ = p.unite(V(0), V(1), 5)
let _ = p.unite(V(1), V(3), 3)
print(p)
print(p)
print(Deque.of([1, 2, 3]))
var l: LinkedList<Int> = LinkedList()
l.push(4)
l.push(5)
print(l)
var b = DynBitset(4)
b.set(1)
b.set(3)
print(b)
print(Subset<V>.single(V(0)) | Subset<V>.single(V(2)))
