// expect: E0203
// rule: 04.2.28
// R04.2.28: a library domain walks in `ord` order but has no neighbours: `.next` belongs to the ordered core domains.
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

print(Two.nth(0).next) // <- here
