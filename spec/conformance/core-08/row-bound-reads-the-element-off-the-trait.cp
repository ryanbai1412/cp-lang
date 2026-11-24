// expect: E0704
// rule: 08.4.1
// A static method's row bound reaches the element through any sequence trait:
// the element is read off the trait's declaration (`Rows<T>: Seq<T>` holds `T`),
// so a user's trait checks the record as `Seq` and `Indexed` do.
trait Rows<T>: Seq<T>
impl<T> Rows<T> for List<T>
struct Pts:
    n: Int
    fn of<S: Rows<R>, R: (u: Int, ..)>(s: S) -> Int = 0
print(Pts.of([(v: 1)])) // <- here
