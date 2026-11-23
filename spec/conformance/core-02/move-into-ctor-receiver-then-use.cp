// expect: E0608 at 10
// rule: 02.1.2
// After `let k = B(n, xs).m()` moves `xs`, reading `xs` again is a use after move.
struct B(n: Int, d: List<Int>):
    fn m(self) -> Int = self.n + self.d.len
fn f() -> Int:
    var xs = List<Int>()
    xs.push(7)
    let k = B(3, xs).m()
    return k + xs.len
print(f())
