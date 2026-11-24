# yac6p3 review

Score: pending
Dream: pending

Cold review: the four corner answers are named for what they are (least x + y, least y - x,
least x - y, greatest x + y) instead of a/b/c/d. One `cell(sum, diff) -> ?Cell` turns a sum and
a difference into a grid cell or None (parity or off the grid), so the hand range check and the
`lower_pair` flag are gone: the fifth question is an `if let ... , near(first) == 0` guard and
both pairings read as cell(sum, diff) twice. Queries take a typed `Cell`.
