// expect: E0302
// engines: next
// rule: 07.8.1
// `Str(x)` makes a word of characters; a user type's text is its `Show`, `"{p}"` (decisions 390, 678).
struct Pair(a: Int, b: Int)
impl Show for Pair:
    fn show(self) -> Str = "<{a},{b}>"
let p = Pair(3, 4)
print(Str(p)) // <- here
