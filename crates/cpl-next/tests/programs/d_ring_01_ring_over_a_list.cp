// expect: ok, output "0 0 0 2\n2 0 1 0\n2\n2\n2\n0 1\n3 2"
// rule: R04.8.11
// A list is `[Int]T`, so it takes `self: [D]T` std fns as a counted array does, and a ring over
// it counts through the list's length, not a `D.count` its `Int` positions lack.
print([5].ring.next(0), [5].ring.prev(0), [1, 2, 3].ring.next(2), [1, 2, 3].ring.step(0, -4))
let r = [1, 2, 3].ring.span(2, 2)
print(r.len, r.end, if r.contains(0): 1 else: 0, if r.contains(1): 1 else: 0)
for w in [4, 5, 6].ring.windows(2): print(w.len)
domain V = 0..3
let av: [V]Int = [V] v => Int(v) * 10
print(av.ring.next(V(2)).ord, if av.ring.span(V(2), 2).contains(V(0)): 1 else: 0)
let xs = [1, 3, 3, 5, 8]
print(xs.first_gt(3)!, xs.last_lt(5)!)
