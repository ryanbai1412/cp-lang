// Closed-form range sums whose terms or partial products pass 64 bits
// while the sum fits: each against the loop it stands for.
let big = 9_223_372_036_854_775_807
fn walk(lo: Int, hi: Int, by: Int) -> Int:
    var s = 0
    for x in (lo..hi).step(by): s += x
    s
assert (big - 1..big).sum() == big - 1
assert (0..big).step(big - 1).sum() == walk(0, big, big - 1)
assert (-big..0).step(big).sum() == walk(-big, 0, big)
assert (-big..-big + 7).step(7).sum() == walk(-big, -big + 7, 7)
assert (-big / 3..-big / 3 + 3).sum() == walk(-big / 3, -big / 3 + 3, 1)
assert (-3_000_000_000..3_000_000_000).sum() == -3_000_000_000
assert (-4_000_000_000..=4_000_000_000).step(2).sum() == 0
print((-3_000_000_000..3_000_000_000).sum(), (1..=4_294_967_295).sum())
