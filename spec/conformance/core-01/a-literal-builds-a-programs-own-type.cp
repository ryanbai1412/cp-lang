// expect: ok, output "12 3 3\n14 4 0"
// rule: 01.8.8
// A list literal and a comprehension build any type with a `ListLiteral` impl, not just `List`.
struct Bag:
    var total: Int
    var n: Int
    var room: Int

impl ListLiteral<Int> for Bag:
    fn with_room(k: Int) -> Bag:
        Bag(0, 0, k)
    fn push(mut self, x: Int):
        total += x
        n += 1

let b: Bag = [3, 4, 5]
let c: Bag = [x * x for x in 0..4]
print(b.total, b.n, b.room)
print(c.total, c.n, c.room)
