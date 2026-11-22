// expect: ok, output "-42 -123 0\n123456789012345678901234567890 0 -7\n265252859812191058636308480000000\n-2148548181881987 107129627388040257"
use std.math.{BigInt, gcd}
print(BigInt.from_int(-42).to_str(), BigInt.from_str("-000123").to_str(), BigInt.from_str("-0").to_str())
print(BigInt.from_str("123456789012345678901234567890").to_str(), BigInt.from_int(0).to_str(), BigInt.from_int(-7).to_str())
assert BigInt.from_int(-7).divmod(BigInt.from_int(3)) == (BigInt.from_int(-3), BigInt.from_int(2))
assert BigInt.from_int(7).divmod(BigInt.from_int(-3)) == (BigInt.from_int(-2), BigInt.from_int(1))
assert BigInt.from_int(-5).abs() == BigInt.from_int(5) and BigInt.gcd(BigInt.from_int(12), BigInt.from_int(-18)) == BigInt.from_int(6) and BigInt.gcd(BigInt.from_int(0), BigInt.from_int(0)) == BigInt.from_int(0)
let vals = [0, 1, -1, 7, -7, 999_999_999, -1_000_000_000, 1_000_000_001, 123_456_789_012, -987_654_321_098, 3_000_000_000_000_000_000, -3_037_000_499]
for x in vals:
    let bx = BigInt.from_int(x)
    assert bx.to_str() == x.to_str() and BigInt.from_str(x.to_str()) == bx and -bx == BigInt.from_int(-x)
    for y in vals:
        let by = BigInt.from_int(y)
        assert (bx < by) == (x < y)
        if abs(x) < 4_000_000_000_000_000_000 and abs(y) < 4_000_000_000_000_000_000:
            assert bx + by == BigInt.from_int(x + y) and bx - by == BigInt.from_int(x - y)
        if abs(x) <= 3_037_000_499 and abs(y) <= 3_037_000_499: assert bx * by == BigInt.from_int(x * y)
        if y != 0:
            assert bx.divmod(by) == (BigInt.from_int(x / y), BigInt.from_int(x % y))
            assert bx / by == BigInt.from_int(x / y)
        assert BigInt.gcd(bx, by) == BigInt.from_int(gcd(x, y))
var f = BigInt.from_int(1)
for i in 1..=30: f = f * BigInt.from_int(i)
print(f.to_str())
let (q, r) = f.divmod(BigInt.from_str("-123456789123456789"))
print(q.to_str(), r.to_str())
assert q * BigInt.from_str("-123456789123456789") + r == f
