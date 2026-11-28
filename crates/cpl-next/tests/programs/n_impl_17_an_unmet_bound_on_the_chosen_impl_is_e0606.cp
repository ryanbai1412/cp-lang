// expect: E0606
// rule: diagnostics E0606
// The impl chosen by its head must have its bounds met: `List<Bool>` takes the `List<T: Show>` impl, which needs `Bool: Show`, so it does not fall back to the blanket impl (R08.1.9).
trait Name:
    fn name(self) -> Int
impl<T: Show> Name for List<T>:
    fn name(x) = 1
impl<T> Name for T:
    fn name(x) = 2
fn via<T: Name>(x: T) -> Int = x.name()
print(via([true]))  // <- here
