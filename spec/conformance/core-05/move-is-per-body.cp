// expect: ok, output "3 3"
// rule: 05.7.6
// A move is a fact about one body: `make` moving its local `xs` leaves the field `xs` usable in `len`.
struct Bag:
    xs: List<Int>
    pub fn make(n: Int) -> Bag:
        var xs: List<Int> = []
        for i in 0..n: xs.push(i)
        Bag(xs)
    pub fn len(self) -> Int = xs.len
let b = Bag.make(3)
print(b.len(), b.xs.len)
