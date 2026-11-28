// expect: E0203
// rule: 08.1.2
// tool: check
// has: "`Sized` has no fn `totl`"
// has: "`Sized`'s member is `total`"
// A misspelled member would be a helper no call reaches while the default runs, so it is an error.
trait Sized:
    fn total(self) -> Int = 0
struct Box3(a: Int, b: Int, c: Int)
impl Sized for Box3:
    fn totl(self) -> Int = a + b + c // <- here
print(Box3(1, 2, 3).total())
