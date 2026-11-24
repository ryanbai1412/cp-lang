// expect: 1
// rule: 02.1.1
// A `()` field given to a fn value is a value like any other: the closure takes it and the call builds in C++.
struct Slot<K, V>:
    var key: K
    var val: V
struct Tree<K, V>:
    var slots: List<Slot<K, V>>
    fn find(self, ok: (K, V) -> Bool) -> Int:
        for i in 0..slots.len:
            if ok(slots[i].key, slots[i].val): return i
        -1
struct Bag<K>:
    var t: Tree<K, ()>
    fn find(self, ok: K -> Bool) -> Int = t.find((k, _) => ok(k))
let b = Bag(Tree<Int, ()>([Slot(1, ()), Slot(5, ())]))
print(b.find(x => x > 2))
