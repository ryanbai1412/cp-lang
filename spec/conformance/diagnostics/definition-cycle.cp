// expect: E0209
// A domain sized by a call of `f` is one of `f`'s parameter types: one error, at that type.
domain V = 0..3
struct S<B>():
    x: Int
impl Index for S<B>:
    let count = f(B.count)
    @get fn ord(self) -> Int = x
    fn nth(k: Int) -> S<B> = S(k)
fn f(n: Int, x: ?S<V> = None) -> Int = n // <- here
print(f(1))
