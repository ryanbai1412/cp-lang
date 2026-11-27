// expect: ok, output "33 3\n11"
// rule: R07.1.1 R08.2.1
// A generator method of a type parameter's bound is walked by `for`, each instance running its
// impl's body; its element type reads the bound's arguments.
trait Nx<T>:
    fn out(self, u: T) -> yields T
struct L(n: Int)
struct One(s: Str)
impl Nx<Int> for L:
    fn out(self, u: Int) -> yields Int:
        for i in 0..self.n: yield i + u
impl Nx<Str> for One:
    fn out(self, u: Str) -> yields Str:
        yield self.s + u
fn sum<G: Nx<Int>>(g: G, at: Int) -> Int:
    var t = 0
    for a in g.out(at): t += a
    t
fn width(g: Nx<Str>) -> Int:
    var t = 0
    for a in g.out("ab"): t += a.len
    t
print(sum(L(3), 10), width(One("x")))
print(sum(L(2), 5))
