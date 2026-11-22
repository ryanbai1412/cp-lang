// The midpoint of a binary search, with a planted overflow at the top of
// the range: `(lo + hi) / 2` passes `Int.MAX` when both ends are large.
let (lo, hi): (0..=9e18, 0..=9e18) = read()

fn mid(lo: Int, hi: Int) -> Int = (lo + hi) / 2

@alt(mid)
fn mid_slow(lo: Int, hi: Int) -> Int = Int((I128(lo) + I128(hi)) / 2)

print(mid(lo, hi))
