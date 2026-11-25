// expect: E0604
// s.at(1) = s.at(0)-like: the right side reads the struct that is lent on the left.
struct Sums:
    a: List<Int>
    fn at(mut self, i: Int) -> lends mut Int:
        var x = a[i]
        lend mut x
        a[i] = x
    fn get(self, i: Int) -> Int:
        a[i]
fn go() -> Int:
    var s = Sums([1, 2, 3])
    s.at(1) = s.get(0) + 10  // <- here
    s.at(2) += s.get(1)
    s.a[1] * 100 + s.a[2]
print(go())
