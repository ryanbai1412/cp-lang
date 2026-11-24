// expect: E0606
// rule: 07.2.18
// `.len` belongs to `Sized`: on a struct view that implements `Seq` but not `Sized`, the error names the missing trait, as for any std view.
struct Evens(n: Int)
impl Seq<Int> for Evens:
    fn iter(self) -> yields Int:
        for i in 0..n:
            if i % 2 == 0: yield i
print(Evens(7).len) // <- here
