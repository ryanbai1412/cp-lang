// expect: ok, output "1 1\n2 2"
// An impl is chosen by its head's rank, the most specific first (R08.1.9), whether called directly or through a bound: `List<Int>` takes the `List<T: Show>` impl, and `Int` the blanket one.
trait Name:
    fn name(self) -> Int
impl<T: Show> Name for List<T>:
    fn name(x) = 1
impl<T> Name for T:
    fn name(x) = 2
fn via<T: Name>(x: T) -> Int = x.name()
print([1, 2].name(), via([1]))
print(5.name(), via(5))
