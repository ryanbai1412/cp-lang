// expect: E0302
// rule: R02.5.5
// engines: next
// has: "a span is one value, not 2 names"
// has: "write `.bounds()`"
// Any library says why its result can't be taken apart; `{n}` is the pattern's count.
struct Span(lo: Int, hi: Int):
    fn bounds(self) -> (Int, Int) = (lo, hi)
@on_destructure("a span is one value, not {n} names", fix: ".bounds()")
fn widest() -> Span = Span(1, 4)
let (lo, hi) = widest() // <- here
print(lo, hi)
