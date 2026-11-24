// expect: ok, output "10 100 6\n4 6 8 10\n0 4 8\n3 2 2\n1 2 3 2 7"
// rule: 06.3.6
// A `yields T` parameter takes any `Seq<T>`: a list, a map or filter view,
// or a user struct with an `impl Seq<T>`, walked through its `iter`; a
// generic `T` binds to the element, so `tally` sees `Str`s and `Int`s alike,
// and `heads` binds `T` through a generic struct's walk.
struct Evens(n: Int)
impl Seq<Int> for Evens:
    fn iter(self) -> yields Int:
        for i in 0..n: yield 2 * i
struct Rows<T>(xs: List<T>)
impl Seq<List<T>> for Rows<T>:
    fn iter(self) -> yields List<T>:
        for x in xs: yield [x, x]
fn heads<T>(self: yields List<T>) -> List<T>:
    var out: List<T> = []
    for r in self: out.push(r[0])
    out
fn total(self: yields Int) -> Int:
    var s = 0
    for x in self: s += x
    s
fn doubled(self: yields Int) -> yields Int:
    for x in self: yield 2 * x
fn tally<T>(self: yields T) -> Int:
    var c = 0
    for _ in self: c += 1
    c
let a = [1, 2, 3, 4]
print(a.total(), a.map(x => x * 10).total(), a.filter(x => x % 2 == 0).total())
print(a.map(x => x + 1).doubled().collect())
print(Evens(3).doubled().collect())
print(["x", "y", "z"].tally(), a.filter(x => x > 2).tally(), Evens(2).tally())
print(Rows([1, 2, 3]).heads(), Rows(["a", "b"]).tally(), heads(Rows([7])))
