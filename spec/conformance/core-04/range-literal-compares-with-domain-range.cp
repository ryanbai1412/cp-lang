// expect: ok, output "YES"
// rule: 04.4.30
// A range literal compared with a `Range<I>` takes its type from the other side, as a list
// literal does, so its ends are positions of `I`.
domain I = 0..3
fn span(n: Int) -> Range<I> = I.nth(0)..I.nth(n)
print(yes(span(2) == 0..2))
