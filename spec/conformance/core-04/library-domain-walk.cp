// expect: ok, output "6 1 2 3 12 13 23 0 1 2 3 4 5"
// rule: 04.2.26 04.2.27
// R04.2.27: a struct with `impl Index` is a domain: `[T]E` and `for x in T` walk `T.nth(0)`, `T.nth(1)`, …, and `T.nth(k).ord` is `k`.
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

var seen: [Two]Int = 0
for t in Two: seen[t] = t.a * 10 + t.b
print(Two.count, [seen[Two.nth(k)] for k in 0..6], [Two.nth(k).ord for k in 0..6])
