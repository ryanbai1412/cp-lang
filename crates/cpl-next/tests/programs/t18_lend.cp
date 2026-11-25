// expect: R0401
// input: "4\n"
// A lender: the caller's write runs in the middle, then the rest of the lender repairs its totals.
struct Sums:
    a: List<Int>
    var total: Int
    var writes: Int

    fn at(mut self, i: Int) -> lends mut Int:
        assert 0 <= i and i < a.len, "index out of range"  // <- here
        var x = a[i]
        lend mut x
        a[i] = x
        var t = 0
        for v in a: t += v
        total = t
        writes += 1

let n: Int = read()
var init: List<Int> = []
for i in 0..n: init.push(i)
var s = Sums(init, 0, 0)
s.at(1) = 40
print("{s.total} {s.writes}")
s.at(0) += 5
print("{s.total} {s.writes}")
for i in 0..n:
    if i == 2: continue
    s.at(i) += i * i
print("{s.total} {s.writes}")
s.at(n) = 1
