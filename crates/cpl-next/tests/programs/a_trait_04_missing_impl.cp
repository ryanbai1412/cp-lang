// expect: E0606
// rule: R08.1.2
trait Get:
    fn get(self) -> Int
fn use_it<T: Get>(x: T) -> Int = x.get()
print(use_it(3)) // <- here
