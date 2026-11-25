// expect: E0604
// Compound assign whose value expression mutates the struct being lent.
struct Sums:
    a: List<Int>
    fn at(mut self, i: Int) -> lends mut Int:
        var x = a[i]
        lend mut x
        a[i] = x
    fn grow(mut self) -> Int:
        a.push(100)
        a.len
fn go() -> Int:
    var s = Sums([1, 2, 3])
    s.at(1) += s.grow()  // <- here
    s.a[1] * 10 + s.a.len
print(go())
