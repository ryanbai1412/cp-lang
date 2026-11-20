//! A non-negative number below 1e36, for exact answers one multiplication
//! past `Int`. The cheat sheet has `I128` as a core type; the reference
//! interpreter has only the trapping `Int`, so this is two base-1e18
//! halves with the few operations noip20p1 and noip19p5 need
//! (friction-backlog-batch-29-nt-linalg, O1).
let base = 1_000_000_000_000_000_000

/// `hi * 1e18 + lo`. The small factors and divisors are 1 to 9, so that
/// `lo * k` and `rem * 1e18 + lo` stay inside `Int`.
pub struct Wide(hi: Int, lo: Int):
    pub fn of(x: Int) -> Wide = Wide(x / base, x % base)

    /// `x * x` for `|x|` below 3e18.
    pub fn square(x: Int) -> Wide:
        let half = 1_000_000_000
        let (a, b) = (abs(x) / half, abs(x) % half)
        let cross = 2 * a * b
        let low = cross % half * half + b * b
        Wide(a * a + cross / half + low / base, low % base)

    pub fn plus(self, y: Wide) -> Wide:
        let sum = lo + y.lo
        Wide(hi + y.hi + sum / base, sum % base)

    pub fn times(self, k: Int) -> Wide:
        let low = lo * k
        Wide(hi * k + low / base, low % base)

    /// The remainder of a division by `k`.
    pub fn rem(self, k: Int) -> Int = (hi % k * (base % k) + lo) % k

    /// The quotient of a division by `k`, rounded down.
    pub fn over(self, k: Int) -> Wide:
        let low = hi % k * base + lo
        Wide(hi / k, low / k)

    pub fn show(self) -> Str:
        if hi == 0: "{lo}" else: "{hi}{lo:018}"

@test
fn carries_between_the_halves():
    let x = Wide.of(999_999_999_999_999_999).times(9)
    assert x == Wide(8, 999_999_999_999_999_991)
    assert x.plus(Wide.of(9)) == Wide(9, 0)
    assert x.rem(7) == 8_999_999_999_999_999_991 % 7
    assert x.over(9) == Wide.of(999_999_999_999_999_999)
    assert Wide(9, 0).over(4).show() == "2250000000000000000"
    assert Wide.square(-3_000_000_007) == Wide(9, 42_000_000_049)
    assert Wide.square(2_999_999_999_999_999_999).show() == "8999999999999999994000000000000000001"
    assert Wide(3, 5).show() == "3000000000000000005"
