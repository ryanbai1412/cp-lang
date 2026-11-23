// expect: ok, output "2 3 6\n0 5 5 4 3 2 1 0"
// R04.2.27: a library domain is the `Seq` of its walk: `T.find`, `T.count(p)`, `T.first`, `T.last` and `T.rev`.
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

print(Two.find(t => t.b == 3)!.ord, Two.count(t => t.a == 0), Two.count)
print(Two.first!.ord, Two.last!.ord, [t.ord for t in Two.rev])
