// expect: ok, output "9 3 12"
// A top-level value lent to a small inlined method moves straight between its words and the
// method's, both into a `mut self` call and into a lender's region body.
struct Acc:
    var xs: List<Int>
    fn add(mut self, x: Int):
        xs.push(x)
    fn at(mut self, i: Int) -> lends mut Int:
        var cell = xs[i]
        lend cell
        xs[i] = cell
var a = Acc([])
for i in 1..=3: a.add(i)
a.at(2) *= 2
var s = 0
for x in a.xs: s += x
print(s, a.xs.len, s + a.xs.len)
