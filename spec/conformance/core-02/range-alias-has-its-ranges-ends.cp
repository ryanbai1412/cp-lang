// expect: ok, output "-10 10 21\n-10 10"
// A range alias has its range's `.MIN`, `.MAX` and `.count`, the same ones a generic `T` bound to
// it reads (R02.6.2, decision 536).
type M = -10..=10
print(M.MIN, M.MAX, M.count)
fn ends<T>() -> (Int, Int) = (Int(T.MIN), Int(T.MAX))
let (lo, hi) = ends<M>()
print(lo, hi)
