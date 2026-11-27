// expect: E0605
// rule: R04.2.26
// An impl's `let` is frozen where the impl is declared, so it reads no `var`.
var n = 5
struct Id(i: Int)
impl Index for Id:
    let count = n + 1 // <- here
    @get fn ord(self) -> Int = i
    fn nth(k: Int) -> Id = Id(k)
print(Id.count)
