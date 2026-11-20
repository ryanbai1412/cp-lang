//! Exact decimals: money, and coordinates given with a fixed number of digits after the
//! point. A `F64` reads `0.1` as something slightly different and makes ties and equal
//! distances a matter of luck; a `Fixed<D>` is an `Int` counting units of `10 ** -D`.

/// A decimal with exactly `D` digits after the point, stored as `raw = value * 10 ** D`.
/// `+ -` and comparisons are exact and as fast as on `Int`. `*` and `/` give the exact
/// result rounded down to `D` digits, and need `raw * raw` (for `/`: `raw * 10 ** D`) to
/// fit in an `Int`.
pub struct Fixed<D>(raw: Int):
    /// How many `raw` units make 1.
    let unit = 10 ** D

    /// The whole number `n`.
    pub fn of(n: Int) -> Fixed<D> = Fixed(n * unit)

    /// `num / den`, rounded down to `D` digits.
    pub fn ratio(num: Int, den: Int) -> Fixed<D>:
        assert den != 0
        let s = if den < 0: -1 else: 1
        Fixed(s * num * unit / (s * den))

    /// The decimal written in `s`: an optional sign, digits, and at most `D` digits
    /// after an optional point (`12`, `-0.5`, `+3.`, `.25`). `none` for anything else,
    /// including more digits than `D` can hold.
    pub fn parse(s: Str) -> ?Fixed<D>:
        var i = 0
        let neg = s.len > 0 and s[0] == '-'
        if s.len > 0 and (s[0] == '-' or s[0] == '+'): i = 1
        var (acc, digits, after) = (0, 0, 0)
        var point = false
        while i < s.len:
            if s[i] == '.' and not point:
                point = true
            elif s[i].is_digit and (not point or after < D):
                acc = acc * 10 + (s[i].ord - '0'.ord)
                digits += 1
                if point: after += 1
            else:
                return none
            i += 1
        if digits == 0: return none
        acc *= 10 ** (D - after)
        Fixed(if neg: -acc else: acc)

    /// The largest whole number that is `<= self`, and the smallest that is `>= self`.
    pub @get fn floor(self) -> Int = raw / unit
    pub @get fn ceil(self) -> Int = -((-raw) / unit)

    /// The nearest whole number; a half rounds up.
    pub @get fn round(self) -> Int = (2 * raw + unit) / (2 * unit)

    /// The part after the point, in `0..1`: `self - floor`.
    pub @get fn frac(self) -> Fixed<D> = Fixed(raw % unit)

    pub @get fn abs(self) -> Fixed<D> = Fixed(if raw < 0: -raw else: raw)

    /// The closest `F64`, for a final `sqrt` or a division that need not be exact.
    pub @get fn to_float(self) -> F64 = raw * 1.0 / unit

    /// `self * k` for a whole `k`: exact, with no rounding and no `raw * raw`.
    pub fn times(self, k: Int) -> Fixed<D> = Fixed(raw * k)

impl Add for Fixed<D>:
    fn add(a, b) = Fixed(a.raw + b.raw)

impl Sub for Fixed<D>:
    fn sub(a, b) = Fixed(a.raw - b.raw)

impl Neg for Fixed<D>:
    fn neg(a) = Fixed(-a.raw)

impl Mul for Fixed<D>:
    fn mul(a, b) = Fixed(a.raw * b.raw / 10 ** D)

impl Div for Fixed<D>:
    fn div(a, b):
        assert b.raw != 0
        let s = if b.raw < 0: -1 else: 1
        Fixed(s * a.raw * 10 ** D / (s * b.raw))

impl Ord for Fixed<D>:
    fn lt(a, b) = a.raw < b.raw

impl Show for Fixed<D>:
    /// Always `D` digits after the point: `Fixed<2>.of(3)` shows as `3.00`.
    fn show(a):
        let mag = if a.raw < 0: -a.raw else: a.raw
        var out = if a.raw < 0: "-" else: ""
        out = out + "{mag / 10 ** D}"
        if D == 0: return out
        let tail = "{mag % 10 ** D}"
        out.push('.')
        for _ in tail.len..D: out.push('0')
        out + tail

/// Random `Fixed<D>` values survive being shown and parsed back.
fn check_round_trip<D>():
    type F = Fixed<D>
    for _ in 0..100:
        let x = F(rand(-10 ** 9, 10 ** 9))
        let text = "{x}"
        assert F.parse(text) == x
        // the digits are the raw value with a point put in
        let plain = [c for c in text if c != '.' and c != '-']
        var back = 0
        for c in plain: back = back * 10 + (c.ord - '0'.ord)
        assert back == x.abs.raw and plain.len >= D + 1
        assert (text[0] == '-') == (x.raw < 0)

@test
fn show_and_parse_round_trip():
    // the digit count is a compile-time parameter, so each width is its own call
    check_round_trip::<0>()
    check_round_trip::<1>()
    check_round_trip::<2>()
    check_round_trip::<3>()
    check_round_trip::<6>()

@test
fn parse_accepts_decimals_and_nothing_else():
    type F = Fixed<2>
    assert F.parse("12.5") == F(1250) and F.parse("12.50") == F(1250) and F.parse("12") == F(1200)
    assert F.parse("-0.05") == F(-5) and F.parse("+3.") == F(300) and F.parse(".25") == F(25)
    assert F.parse("007.10") == F(710) and F.parse("-0") == F(0)
    for bad in ["", "-", ".", "1.234", "1.2.3", "1e5", "--1", "1,5", " 1", "1.-2", "abc"]:
        assert F.parse(bad) == none
    assert "{F(1250)}" == "12.50" and "{F(-5)}" == "-0.05" and "{F.of(3)}" == "3.00"
    assert "{Fixed<0>.of(-42)}" == "-42" and "{Fixed<4>(1)}" == "0.0001"

/// `+ - * /` and the rounding accessors of `Fixed<D>` agree with exact fractions.
fn check_arithmetic<D>():
    // value = raw / 10 ** d as a pair (numerator, denominator); cross-multiply to compare
    type F = Fixed<D>
    let u = 10 ** D
    for _ in 0..300:
        let (a, b) = (F(rand(-5000, 5000)), F(rand(-5000, 5000)))
        assert (a + b).raw == a.raw + b.raw and (a - b).raw == a.raw - b.raw and (-a).raw == -a.raw
        assert (a < b) == (a.raw < b.raw) and a.times(7) == a + a + a + a + a + a + a
        // a * b is the largest multiple of 1 / u that is <= the exact product
        let p = (a * b).raw
        assert p * u <= a.raw * b.raw and a.raw * b.raw < (p + 1) * u
        if b.raw != 0:
            // the same for a / b: q / u <= a / b < (q + 1) / u
            let q = (a / b).raw
            let s = if b.raw < 0: -1 else: 1
            assert q * s * b.raw <= s * a.raw * u and s * a.raw * u < (q + 1) * s * b.raw
            assert F.ratio(a.raw, b.raw) == a / b
        assert a.floor * u <= a.raw and a.raw < (a.floor + 1) * u
        assert (a.ceil - 1) * u < a.raw and a.raw <= a.ceil * u
        assert abs(2 * a.raw - 2 * a.round * u) <= u and F.of(a.floor) + a.frac == a
        assert abs(a.to_float * u - a.raw) < 0.001

@test
fn arithmetic_matches_exact_fractions():
    check_arithmetic::<0>()
    check_arithmetic::<1>()
    check_arithmetic::<2>()
    check_arithmetic::<4>()

@test
fn splitting_a_bill_loses_no_cent():
    // share a total among n people in whole cents: everyone pays the rounded-down
    // share, and the first few pay one cent more
    type Money = Fixed<2>
    for _ in 0..50:
        let total = Money(rand(0, 100000))
        let n = rand(1, 9)
        let share = total / Money.of(n)
        let extra = (total - share.times(n)).raw
        assert 0 <= extra and extra < n
        var paid = Money.of(0)
        for i in 0..n: paid = paid + (if i < extra: share + Money(1) else: share)
        assert paid == total
    let third = Money.parse("10")! / Money.of(3)
    assert "{third}" == "3.33"
