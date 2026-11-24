// expect: ok
// rule: 06.4.3
// tool: stress --seed 1 --cases 50
// has: "cases agree"
// An `impl Random` over a generic alias draws the alias: `gen<Order<V>>()` is `Order<V>.gen()`.
domain V = 0..5
type Order<W> = [W]W
impl<W> Random for Order<W>:
    fn gen(fixed: Bool = false) -> Order<W>:
        var p: [W]W = [W] w => w
        if not fixed: shuffle(mut p)
        p
@gen fn input():
    let p = gen<Order<V>>()
    let q = gen<Order<V>>(fixed: true)
    assert p.sorted() == V.collect()
    assert V.all(v => q[v] == v)
    print(p.len)
@alt(program)
fn brute():
    let k: Int = read()
    print(k)
let k: Int = read()
print(k)
