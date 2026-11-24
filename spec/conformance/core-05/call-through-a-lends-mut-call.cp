// expect: ok, output "101 4 5 15 3 7"
// rule: 05.6.3
// A `mut self` method called on a place under a `lends mut` call changes the arena's slot: a field, tuple index or element, from top level or inside a method.
use std.marks.{Arena, Id}
struct C:
    var n: Int
    fn bump(mut self, by: Int):
        n += by
struct Node:
    var kids: List<Int>
    var c: C
    var pair: (C, Int)
    var heap: MinHeap<Int>
struct Pool:
    var a: Arena<Node>
    fn grow(mut self, i: Id<Node>, x: Int):
        a.at(i).kids.push(x)
        a[i].c.n += x
        a.at(i).c.bump(1)
        a[i].heap.push(x - 1)
var p = Pool(Arena<Node>())
let i = p.a.alloc(Node([1], C(0), (C(5), 0), MinHeap<Int>()))
p.grow(i, 4)
p.a.at(i).kids.push(9)
p.a.at(i).kids.pop()
p.a[i].pair.0.bump(10)
p.a.at(i).kids[0] += 100
p.a.at(i).heap.push(7)
print(p.a.at(i).kids, p.a.at(i).c.n, p.a.at(i).pair.0.n, p.a.at(i).heap.pop() ?? -1, p.a[i].heap.pop() ?? -1)
