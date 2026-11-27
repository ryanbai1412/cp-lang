// expect: ok, output "6 5 0 1 2 3 4 5 1 6"
// rule: R04.2.26
// rule: R04.2.27
// A struct is a domain by implementing std's `Index`: `let count` meets the trait's `count`, and
// `[T]E`, `for t in T`, `T.count`, `T.nth` and a `D: Domain` bound all work. `Two(a, b)` is still
// its constructor (R04.2.28).
struct Two(a: Int, b: Int)
impl Index for Two:
    let count = 6
    @get fn ord(self) -> Int = a * (7 - a) / 2 + b - a - 1
    fn nth(k: Int) -> Two:
        var a = 0
        var r = k
        while r >= 3 - a:
            r -= 3 - a
            a += 1
        Two(a, a + 1 + r)
fn size<D: Domain>() -> Int:
    var n = 0
    for d in D: n += 1
    n
var seen: [Two]Int = 0
for t in Two: seen[t] = t.ord
print(Two.count, Two(2, 3).ord, [seen[Two.nth(k)] for k in 0..Two.count], Two.nth(4).a, size<Two>())
