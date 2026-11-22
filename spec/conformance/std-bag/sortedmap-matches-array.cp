// expect: ok, output "0 YES\n2 1 a 3 c a none YES NO\n3 c none 1 a none 1 2 3 c none\n1 a\n3 c 3 c\n1 a 1 3 a c\nYES a\n4 40 4 40 4 40 2\n1 10 6 60 0 none none\n5 7 YES\n1 2\n5 3\nYES"
use std.bag.{SortedMap}
use std.rand.{Rng}
fn show<T>(o: ?T) -> Str = match o:
    Some(x) => "{x}"
    None => "none"
var m: SortedMap<Int, Str> = SortedMap()
print(m.len, m.is_empty)
m.set(3, "c")
m.set(1, "a")
m.set(1, "b")
m.set(1, "a")
print(m.len, show(m.first), show(m.last), show(m.get(1)), show(m.get(2)), m.contains(3), m.contains(2))
print(show(m.first_ge(2)), show(m.first_gt(3)), show(m.last_le(2)), show(m.last_lt(1)), m.rank(3), m.rank(9), show(m.nth(1)), show(m.nth(2)))
print(m.items.collect(), m.each_rev().collect(), m.keys.collect(), m.values.collect())
let t = m.take()
print(m.is_empty, show(t.get(1)))
var n: SortedMap<Int, Int> = SortedMap()
n.set(1, 10)
n.set(4, 40)
n.set(6, 60)
print(show(n.first_true((k, v) => v >= 30)), show(n.last_true((k, v) => k <= 5)), n.drain(after: 1, while: (k, v) => v < 50), n.len)
print(show(n.pop_first()), show(n.pop_last()), n.len, show(n.remove(6)), show(n.remove(1)))
var a: SortedMap<Int, Int> = SortedMap()
a.set(1, 10)
var b: SortedMap<Int, Int> = SortedMap()
b.set(1, 5)
b.set(2, 7)
a.absorb(mut b, join: (x, y) => x - y)
print(show(a.get(1)), show(a.get(2)), b.is_empty)
var c: SortedMap<Int, Int> = SortedMap()
c.set(1, 1)
var d: SortedMap<Int, Int> = SortedMap()
d.set(1, 2)
d.set(5, 3)
c.absorb(mut d)
print(c.items.collect())
var rng = Rng(3)
var z: SortedMap<Int, Int> = SortedMap()
var arr = List.filled(50, -1)
var ok = true
for _ in 0..500:
    let k = rng.int(0, 49)
    if rng.int(0, 2) == 0:
        if z.remove(k) != (if arr[k] >= 0: Some(arr[k]) else: None): ok = false
        arr[k] = -1
    else:
        let v = rng.int(0, 99)
        z.set(k, v)
        arr[k] = v
    var lo: ?(Int, Int) = None
    for j in 0..=k:
        if arr[j] >= 0: lo = Some((j, arr[j]))
    if z.last_le(k) != lo: ok = false
print(ok)
