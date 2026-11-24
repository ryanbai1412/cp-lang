// expect: ok, output "7 9 1"
// rule: 08.1.4
// The more specific `self` wins whatever the declaration order: both later
// overloads beat the generic declared first, on both engines.
fn tot<S: Seq<T>, T>(self: S) -> Int = 1
fn tot(self: Stepped<List<Int>, Int>) -> Int = 7
fn tot(self: Range<Int>) -> Int = 9
let xs: List<Int> = [1, 2, 3]
print(xs.step(2).tot(), (3..9).tot(), "ab".tot())
