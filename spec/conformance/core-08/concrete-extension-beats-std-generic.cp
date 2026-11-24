// expect: ok, output "7"
// rule: 08.1.4
// A concrete `self: Range<Int>` that meets the bound of std's generic `sum<S: Seq<T>>` is more specific than it, so the file's fn is the call.
pub fn sum(self: Range<Int>) -> Int = 7
print((0..3).sum())
