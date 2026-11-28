// expect: ok, output "<1> <2|x> 3 7"
// rule: 02.11.4
// Two structs of one name and different arities are two types; an impl on each is that type's.
struct Pair<T>(a: T)
struct Pair<A, B>(a: A, b: B):
    fn both(self) -> Str where A: Show, B: Show = "{a}|{b}"
impl Show for Pair<T: Show>:
    fn show(self) -> Str = "<{a}>"
impl Show for Pair<A: Show, B: Show>:
    fn show(self) -> Str = "<{self.both()}>"
fn first(p: Pair<Int, Int>) -> Int = p.a
let p = Pair<Int>(1)
let q = Pair<Int, Str>(2, "x")
print("{p} {q}", first(Pair<Int, Int>(3, 4)), Pair<Int, Int>(3, 4).b + p.a + q.a)
