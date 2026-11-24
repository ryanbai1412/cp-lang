// expect: ok, output "3 6 2 6"
// rule: 07.3.5 12.4
// A `.rev` view meets `Sized`, `Seq`, `Bidir` and `Positional` through std's
// `impl ... for Rev<S: ...>` heads, each holding when the source meets its bound.
fn count<S: Sized>(s: S) -> Int = s.len
fn total<S: Seq<Int>>(s: S) -> Int = s.sum()
fn second<P: Positional<T>, T>(p: P) -> T = p.nth(1)
fn back<S: Bidir<Int>>(s: S) -> Int = s.rev.sum()
let r = [1, 2, 3].rev
print(count(r), total(r), second(r), back(r))
