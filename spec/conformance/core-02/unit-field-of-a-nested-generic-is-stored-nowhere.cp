// expect: ok, output "3 1"
// rule: 02.6.3
// A `()` field of a generic struct built inside another generic holds nothing, and a store into it still runs.
struct Slot<K, V>:
    var key: K
    var val: V
    var cnt: Int
struct Tree<K, V>:
    var slots: List<Slot<K, V>>
    fn put(mut self, x: K, v: V):
        slots.push(Slot(x, v, 1))
        slots[0].val = v
struct Bag<K>:
    var t: Tree<K, ()>
    fn add(mut self, x: K) = t.put(x, ())
var b = Bag(Tree<Int, ()>([]))
b.add(3)
print(b.t.slots[0].key, b.t.slots[0].cnt)
