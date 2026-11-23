// expect: E0302 at 6
// rule: 04.2.3
// A range end that is optional is the error's place: `lo` is the `?Pos`, not the range around it.
domain Pos = 0..5
let lo = Pos.last
let lo2 = Pos.first!
fn f(r: Range<Pos>) -> Int = r.len
print(f(lo2..=lo))
