use std.lazy.{Rope}
let n: Int = read()
let reps = 1 << 17
var s = 0
domain V = 0..n

let a: [V]Int = [V] v => v.ord * 7919 % n
for _ in 0..100:
    var r = Rope(a, merge: (x, y) => x + y) // timed: lazy.Rope.new
    s += r.get(V(0))
var rope = Rope(a, merge: (x, y) => x + y)
let pick: (mut Int) -> ?Int = (mut x) => if x < 0: Some(x) else: None // within: lazy.Rope.query_path
for i in 0..reps:
    let p = i * 7919 % n
    rope.apply(V(p / 2)..V(p), 1) // timed: lazy.Rope.apply
    rope.reverse(V(p / 3)..V(p)) // timed: lazy.Rope.reverse
    s += rope.get(V(i * 13 % n)) // timed: lazy.Rope.get
    rope.set(V(i * 17 % n), i % 9) // timed: lazy.Rope.set
    s += rope.query_path(V(i * 19 % n), pick) ?? 0 // timed: lazy.Rope.query_path
for _ in 0..100:
    s += rope.to_array()[V(0)] // timed: lazy.Rope.to_array
print(s)
