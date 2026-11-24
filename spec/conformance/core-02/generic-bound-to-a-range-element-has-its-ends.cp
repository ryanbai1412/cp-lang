// expect: ok, output "1 9 9\n1 9"
// rule: 02.6.2
// A generic `T` bound to a range through a `List<T>` or a `Seq<T>` bound sees the range's ends as `T.MIN`/`T.MAX`, never `Int`'s (decision 538).
fn ends<T>(xs: List<T>) -> (T, T, Int) = (T.MIN, T.MAX, T.count)
fn lo<S: Seq<T>, T>(s: S) -> T = T.MIN
fn hi<S: Seq<T>, T>(s: S) -> T = T.MAX
let a: List<1..=9> = [3]
let (l, h, c) = ends(a)
print(l, h, c)
print(lo(a), hi(a))
