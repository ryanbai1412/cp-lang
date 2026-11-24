// expect: ok, output "-1\n5\n-1\n7"
// A `?S` whose struct has a niche (its first field, an `Int`) is as wide as `S`
// (R03.2.2); passing it to a `?S` param a fn only reads reads it in place.
struct Box(n: Int, items: List<Int>)
struct Holder:
    var b: ?Box = None
fn total(found: ?Box) -> Int:
    if let b = found: return b.n + b.items.len
    -1
fn make(k: Int) -> ?Box = if k > 0: Some(Box(k, [1, 2])) else: None
var h = Holder()
print(total(h.b))
h.b = Some(Box(3, [1, 2]))
print(total(h.b))
print(total(make(0)))
print(total(make(5)))
