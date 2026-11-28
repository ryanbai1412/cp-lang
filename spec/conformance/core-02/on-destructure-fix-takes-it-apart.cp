// expect: ok, output "1 4"
// rule: R02.5.5
// engines: next
// The fix `@on_destructure` offers gives what the pattern takes apart.
struct Span(lo: Int, hi: Int):
    fn bounds(self) -> (Int, Int) = (lo, hi)
@on_destructure("a span is one value, not {n} names", fix: ".bounds()")
fn widest() -> Span = Span(1, 4)
let (lo, hi) = widest().bounds()
print(lo, hi)
