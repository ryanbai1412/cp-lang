// expect: E0606
// rule: 12.4.7
// A user `Seq` without a `contains` of its own has no `Contains`: `x in s` is not a scan.
struct Evens(n: Int)
impl Seq<Int> for Evens:
    fn iter(self) -> yields Int:
        for i in 0..n: yield 2 * i
print(yes(4 in Evens(3))) // <- here
