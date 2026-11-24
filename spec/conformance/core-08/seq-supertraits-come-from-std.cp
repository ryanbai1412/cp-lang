// expect: ok, output "3 6 2"
// rule: 08.1.1 07.3.5 07.3.7
// A bound of `Indexed` meets `Sized`, `Seq` and `Positional` through the supertraits std's
// `trait Indexed` and `trait Positional` declare, the same chain a user trait's supers give.
fn count<S: Sized>(s: S) -> Int = s.len
fn total<S: Seq<Int>>(s: S) -> Int = s.sum()
fn second<P: Positional<T>, T>(p: P) -> T = p.nth(1)
fn via<S: Indexed<Int, Int>>(s: S) -> (Int, Int, Int) = (count(s), total(s), second(s))
let (n, t, b) = via([1, 2, 3])
print(n, t, b)
