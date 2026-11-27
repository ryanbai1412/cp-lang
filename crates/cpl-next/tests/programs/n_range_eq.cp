// expect: ok, output "YES YES\nYES YES\nNO NO"
// rule: R08.1.5 R04.4.30
// Two ranges are equal when they hold the same positions in the same order, however they were
// written; a literal beside a `Range<I>` takes its ends as positions of `I`.
domain I = 0..4
fn span(n: Int) -> Range<I> = I.nth(0)..I.nth(n)
print(yes(span(2) == 0..2), yes((0..3) == (0..=2)))
print(yes((2..2) == (5..5)), yes((1..2 by 3) == (1..=1)))
print(yes((0..4 by 2) == (0..4)), yes(span(1) == 1..2))
