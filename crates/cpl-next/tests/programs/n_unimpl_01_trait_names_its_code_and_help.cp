// expect: E0311
// rule: R08.1.8
// A trait's `@on_unimplemented(code: C, "..{T}..")` makes a missing impl of it report `C`, with
// the message as help and `{T}` the type shown; a type with the impl is unchanged.
@on_unimplemented(code: E0311, "a dense table needs finitely many keys, and `{T}` has no end")
trait Dense:
    fn size() -> Int
impl Dense for Bool:
    fn size() -> Int = 2
fn n<T: Dense>(x: T) -> Int = T.size()
print(n(true))
print(n(5)) // <- here
