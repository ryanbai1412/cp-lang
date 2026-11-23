// expect: ok, output "4 6 6"
// R04.2.27: `D: Domain` takes a core domain and a library domain alike: `for d in D`, `[D]T` and `D.count`.
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

domain V = 0..4

fn size<D: Domain>() -> Int:
    var n = 0
    for d in D: n += 1
    n

fn tally<D: Domain>() -> [D]Int:
    var c: [D]Int = 0
    for d in D: c[d] += D.count
    c

let t = tally<Two>()
print(size<V>(), size<Two>(), t[Two.nth(5)])
