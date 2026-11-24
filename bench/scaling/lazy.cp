// sizes stay at n: each insert is paired with an erase
use std.lazy.{Flip, LazyTreap, Rope, SeqFold, flip_action}
use std.algebra.{Action, add}
let n: Int = read()
let reps = 1 << 17
var s = 0
domain V = 0..n

for i in 0..reps:
    s += Flip.of(i % 2 == 0, i, 0).now // timed: lazy.Flip.of
    s += Flip.op(+).op(Flip(i, 0), Flip(0, i)).now // timed: lazy.Flip.op
    s += flip_action(+).apply(i % 2 == 0, Flip(i, 1), 4).now // timed: lazy.flip_action

fn treap() -> LazyTreap<Int, Int> = LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f)
for i in 0..reps:
    var e = treap() // timed: lazy.LazyTreap.new
    if e.is_empty: s += i % 2
var t = treap()
for i in 0..n: t.push(i * 7919 % n)
var o = LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f, before: (x, y) => x < y)
for i in 0..n: o.insert(i * 7919 % n)
for i in 0..reps:
    let p = i * 7919 % n
    s += t.len // timed: lazy.LazyTreap.len
    if t.is_empty: s += 1 // timed: lazy.LazyTreap.is_empty
    t.apply_all(1) // timed: lazy.LazyTreap.apply_all
    t.apply_range(p / 2, p, 1) // timed: lazy.LazyTreap.apply_range
    t.apply(p / 2..p, -1) // timed: lazy.LazyTreap.apply
    t.reverse(p / 3..p) // timed: lazy.LazyTreap.reverse
    s += t.get(i * 13 % n) // timed: lazy.LazyTreap.get
    t.insert_at(p, i) // timed: lazy.LazyTreap.insert_at
    s += t.erase_at(i * 17 % n) // timed: lazy.LazyTreap.erase_at
    t.push(i) // timed: lazy.LazyTreap.push
    t.erase_range(p..p + 1) // timed: lazy.LazyTreap.erase_range
    o.insert(p) // timed: lazy.LazyTreap.insert
    s += o.erase_at(i * 17 % n)
    var u = t.split_at(t.len - 1 - i % 8) // timed: lazy.LazyTreap.split_at [let: k = n - 4]
    t.splice_back(mut u) // timed: lazy.LazyTreap.splice_back [fixed: m]
    s += t.take(4).len // timed: lazy.LazyTreap.take [fixed: k]
for _ in 0..200:
    s += t.to_list().len // timed: lazy.LazyTreap.to_list
// Storage grows by moving the nodes already there.
for _ in 0..100:
    var r = treap()
    for i in 0..n: r.push(i)
    r.reserve(2 * n) // timed: lazy.LazyTreap.reserve
    s += r.len

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

let xs = [i * 7919 % n for i in 0..n]
let act = Action<Int, Int>.of(add, +)
for _ in 0..8:
    s += SeqFold<Int, Int>(+, xs, act: act).total // timed: lazy.SeqFold.new
var sf = SeqFold<Int, Int>(+, xs, act: act)
for i in 0..reps:
    let p = i * 7919 % n
    s += sf.len // timed: lazy.SeqFold.len
    if sf.is_empty: s += 1 // timed: lazy.SeqFold.is_empty
    s += sf.total // timed: lazy.SeqFold.total
    s += sf.get(i * 13 % n) // timed: lazy.SeqFold.get
    s += sf.fold(p / 2..p) // timed: lazy.SeqFold.fold
    sf.apply(p / 2..p, 1) // timed: lazy.SeqFold.apply
    sf.reverse(p / 3..p) // timed: lazy.SeqFold.reverse
for _ in 0..40:
    for x in sf.items: // timed: lazy.SeqFold.items
        s += x // timed: lazy.SeqFold.items
// 3n / 2 inserts take the sequence from n to 2.5n at every size
for i in 0..n / 2:
    sf.push(i) // timed: lazy.SeqFold.push
    sf.push(i) // timed: lazy.SeqFold.push
    sf.insert_at(i * 7919 % n, i) // timed: lazy.SeqFold.insert_at
let k = n / 4
let ys = [i for i in 0..k]
for _ in 0..80:
    var c = SeqFold<Int, Int>(+, xs, act: act)
    c.insert_many(n / 3, ys) // timed: lazy.SeqFold.insert_many
    c.erase_range(n / 5..n / 5 + k) // timed: lazy.SeqFold.erase_range
    var d = c.split_at(n / 2) // timed: lazy.SeqFold.split_at [let: k = n / 2]
    c.splice(n / 7, mut d) // timed: lazy.SeqFold.splice
    s += c.len
print(s)
