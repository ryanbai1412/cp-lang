// expect: ok, output "6 6"
// `for` over a value bound by the iteration role's trait walks it through that trait's one
// method, so a program's own `impl Seq` walks the same as a std type's (R11.6.1).
struct Twice:
    n: Int
impl Seq<Int> for Twice:
    fn iter(self) -> yields Int:
        yield self.n
        yield self.n
fn total<S: Seq<Int>>(s: S) -> Int:
    var t = 0
    for x in s: t += x
    t
print(total([1, 2, 3]), total(Twice(3)))
