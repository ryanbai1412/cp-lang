//! Exact fractions of `Int`s, always in lowest terms with a positive denominator.

/// The rational number `num / den`. Arithmetic is exact as long as the cross products
/// fit in an `Int`; an overflow stops the program rather than rounding.
use std.math.{gcd}
pub struct Frac(num: Int, den: Int):
    /// `a / b` in lowest terms. `b` must not be zero.
    pub fn new(a: Int, b: Int = 1) -> Frac:
        assert b != 0
        let g = gcd(abs(a), abs(b))
        let s = if b < 0: -1 else: 1
        Frac(s * a / g, s * b / g)

    /// The largest integer not above the fraction.
    pub @get fn floor(self) -> Int = num / den

    /// The smallest integer not below the fraction.
    pub @get fn ceil(self) -> Int = -((-num) / den)

    pub @get fn inv(self) -> Frac = Frac.new(den, num)

impl Add for Frac:
    fn add(a, b) = Frac.new(a.num * b.den + b.num * a.den, a.den * b.den)

impl Sub for Frac:
    fn sub(a, b) = Frac.new(a.num * b.den - b.num * a.den, a.den * b.den)

impl Mul for Frac:
    fn mul(a, b) = Frac.new(a.num * b.num, a.den * b.den)

impl Div for Frac:
    fn div(a, b) = Frac.new(a.num * b.den, a.den * b.num)

impl Ord for Frac:
    fn lt(a, b) = a.num * b.den < b.num * a.den

@test
fn arithmetic_stays_in_lowest_terms():
    let half = Frac.new(1, 2)
    let third = Frac.new(2, -6)
    assert third == Frac(-1, 3)
    assert half + third == Frac(1, 6) and half - third == Frac(5, 6)
    assert half * third == Frac(-1, 6) and half / third == Frac(-3, 2)
    assert third.inv == Frac(-3, 1) and Frac.new(4) == Frac(4, 1)
    assert third < half and not (half < half) and max(half, third) == half

@test
fn rounding_goes_the_right_way_for_negatives():
    for a in -20..=20:
        for b in [-7, -3, -1, 1, 2, 5]:
            let f = Frac.new(a, b)
            assert Frac.new(f.floor) <= f and f < Frac.new(f.floor + 1)
            assert Frac.new(f.ceil - 1) < f and f <= Frac.new(f.ceil)

@test
fn sorting_slopes_exactly():
    var slopes = [Frac.new(1, 3), Frac.new(-1, 2), Frac.new(2, 6), Frac.new(5, 7), Frac.new(0, 9)]
    slopes.sort()
    assert slopes == [Frac(-1, 2), Frac(0, 1), Frac(1, 3), Frac(1, 3), Frac(5, 7)]
