// expect: E0324
// rule: R08.1.10
// `and` short-circuits and no `@operator` trait declares it, so it has no fn item to name.
trait L:
    fn z(self) -> Int
impl L for type_of(and): // <- here
    fn z(self) -> Int = 7
print(1)
