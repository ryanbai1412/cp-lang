// expect: ok, output "1"
// engines: next
// A domain sized by a call of `f` may be one of `f`'s parameter types: typing `f` needs the
// domain, not its count, and the count is read lazily the first time a run needs it.
domain V = 0..3
struct S<B>():
    x: Int
impl Index for S<B>:
    let count = f(B.count)
    @get fn ord(self) -> Int = x
    fn nth(k: Int) -> S<B> = S(k)
fn f(n: Int, x: ?S<V> = None) -> Int = n
print(f(1))
