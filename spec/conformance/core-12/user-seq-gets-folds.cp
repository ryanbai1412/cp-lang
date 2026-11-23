// expect: ok, output "10 5 10"
// rule: 12.4
// A user type with `impl Seq` walks in `for` and gets every generic fold (R12.4.1, decision 326).
struct Bag(n: Int)
impl Seq<Int> for Bag:
    fn iter(self) -> yields Int:
        for i in 0..self.n: yield i
let b = Bag(5)
var t = 0
for x in b: t += x
print(b.sum(), b.count(), t)
