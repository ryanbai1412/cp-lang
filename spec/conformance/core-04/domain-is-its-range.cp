// expect: ok, output "5\nYES"
// rule: 04.2.21
domain Pos = 1..=5
fn tally(ls: Range<Pos>) -> Int = ls.len
print(tally(Pos))
print(yes(tally(Pos) == Pos.count))
