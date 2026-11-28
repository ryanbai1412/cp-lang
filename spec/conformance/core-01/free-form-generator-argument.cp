// expect: ok, output "14"
// rule: 01.10.1
// engines: next
// A generator argument is the receiver, a sequence: `tally(e for x in s)` is the method call on it, as std's folds are.
trait Tally:
    @free_form
    fn tally(self) -> Int
impl<S: Seq<Int>> Tally for S:
    fn tally(self) -> Int:
        var n = 0
        for x in self: n += x
        n
print(tally(x * x for x in 0..4))
