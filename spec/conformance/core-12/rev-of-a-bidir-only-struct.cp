// expect: ok, output "3 2 1\n3 2 1\n1 2 3"
// rule: 12.4.4
// A struct with only `Seq` and `Bidir` impls gets std's `.rev`: the `Rev` view walks
// its `iter_rev`, shows like any sequence, and reversing it again walks `iter`.
use std.seq.{Seq, Bidir}
struct W(xs: List<Int>)
impl Seq<Int> for W:
    fn iter(self) -> yields Int:
        for x in self.xs: yield x
impl Bidir<Int> for W:
    fn iter_rev(self) -> yields Int:
        for x in self.xs.rev: yield x
print(W([1, 2, 3]).rev.collect())
print(W([1, 2, 3]).rev)
print(W([1, 2, 3]).rev.rev.collect())
