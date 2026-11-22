struct Reg:
    var ids: Map<Str, Int>
    var p: Int

    fn late(mut self) -> Int:
        let f = () => p
        p += 1
        f()

    fn step(mut self, k: Int) -> Int:
        p += k
        p

    fn bump(mut self) -> Int:
        let inc = () => self.step(1)
        let a = inc()
        let b = inc()
        a * 100 + b * 10 + p

struct Pair:
    var left: Reg
    var right: Reg

fn touch(mut r: Reg) -> Int:
    let before = () => r.p
    r.p += 5
    before() + r.late()

var r = Reg(Map<Str, Int>(), 0)
print("p", r.late(), r.bump(), touch(mut r), r.p)
var pair = Pair(Reg(Map<Str, Int>(), 0), Reg(Map<Str, Int>(), 10))
print(pair.left.late(), pair.right.bump(), touch(mut pair.right), pair.left.p, pair.right.p)
