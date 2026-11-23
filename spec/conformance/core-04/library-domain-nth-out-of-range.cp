// expect: R0001 at 16 (checked)
// rule: 04.2.27
// input: "6\n"
// R04.2.27: `T.nth(k)` outside `0..T.count` traps; the impl's `nth` never sees it.
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

let k: Int = read()
print(Two.nth(k).ord)
