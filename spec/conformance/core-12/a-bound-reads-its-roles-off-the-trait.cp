// expect: ok, output "7\n6\n9"
// rule: 12.4
// A sequence bound's key and element are read off the trait's declaration: the
// parameter a method yields is the element, and a super passes its roles on
// through the args it is written with, so a user's `Walk<T>` and a `Table<T>`
// over std's `Positional<T>` bind `U` just as `Indexed<K, U>` does.
trait Walk<T>:
    fn walk(self) -> yields T
struct Bag:
    xs: List<Int>
impl Walk<Int> for Bag:
    fn walk(self) -> yields Int:
        for x in self.xs: yield x
fn first_walked<S: Walk<U>, U>(s: S) -> ?U:
    for x in s.walk(): return x
    None
print(first_walked(Bag(xs: [7, 8]))!)
trait Table<T>: Positional<T>
impl Table<Int> for List<Int>
fn mid<S: Table<U>, U>(s: S) -> U = s.nth(s.len / 2)
print(mid([5, 6, 7]))
fn at<S: Indexed<K, U>, K, U>(s: S, k: K) -> U = s.at(k)
print(at([8, 9], 1))
