// expect: ok, output "0"
// A fn-local domain whose length is fixed by its caller's domain (`0..V.count + 2`) is held as a
// constant, and saving and restoring it around the call still builds.
use std.flow.max_closure
domain V = 0..2
let before: [V]List<V> = [[1], []]
let p = [-2, 2]
let c = max_closure(before, profit: v => p[v.ord])
assert c.take == [false, false]
print(c.value)
