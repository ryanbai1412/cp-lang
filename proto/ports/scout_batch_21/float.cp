//! What the ten ports needed from `Float` and could not get from the
//! reference interpreter: fixed-point printing, comparison with a
//! tolerance, and sine and cosine. Plain userland code, so it is also the
//! proposal: see E1 to E4 of research/friction-backlog-batch-21.md.

pub const PI = 3.141592653589793

/// `x` with exactly `digits` decimals, rounded half away from zero:
/// `fixed(2.5, 0)` is "3", `fixed(-0.004, 2)` is "-0.00" printed as "0.00".
/// Never an exponent, never "-0". `x` must be finite and `|x| * 10^digits`
/// must fit in an `Int`.
pub fn fixed(x: Float, digits: Int) -> Str:
    assert x == x and abs(x) < Float.MAX    // not NaN, not infinite
    var scale = 1
    for _ in 0..digits: scale *= 10
    let units = Int(abs(x) * scale + 0.5)
    var out = Str()
    if x < 0.0 and units > 0: out.push('-')
    for c in "{units / scale}": out.push(c)
    if digits > 0:
        out.push('.')
        var frac = units % scale
        for _ in 0..digits:
            scale /= 10
            out.push(Char('0'.ord + frac / scale))
            frac %= scale
    out

/// -1, 0 or 1: the sign of `x`, where anything within `eps` of zero is zero.
pub fn sign(x: Float, eps: Float) -> Int = if x > eps: 1 elif x < -eps: -1 else: 0

/// Whether `a` and `b` differ by at most `eps`. False when either is NaN.
pub fn close(a: Float, b: Float, eps: Float) -> Bool = abs(a - b) <= eps

/// Sine and cosine of `x` radians together, to about 1e-15.
pub fn sin_cos(x: Float) -> (sin: Float, cos: Float):
    // Halve the angle until the series converges in a few terms, then
    // double it back with the double-angle formulas.
    var (t, halvings) = (x, 0)
    while abs(t) > 0.125:
        t /= 2
        halvings += 1
    var (s, c, term) = (0.0, 0.0, 1.0)      // term = t^k / k!
    for k in 0..14:
        if k % 4 == 0: c += term
        elif k % 4 == 1: s += term
        elif k % 4 == 2: c -= term
        else: s -= term
        term = term * t / (k + 1)
    for _ in 0..halvings: (s, c) = (2 * s * c, c * c - s * s)
    (sin: s, cos: c)

@test
fn fixed_rounds_and_pads():
    assert fixed(3.14159, 2) == "3.14"
    assert fixed(2.5, 0) == "3"
    assert fixed(0.05, 1) == "0.1"
    assert fixed(-1.005, 1) == "-1.0"
    assert fixed(-0.004, 2) == "0.00"
    assert fixed(1e-7, 6) == "0.000000"
    assert fixed(1234567.0, 3) == "1234567.000"
    assert fixed(0.999999, 3) == "1.000"

@test
fn sign_and_close_take_a_tolerance():
    assert sign(1e-12, 1e-9) == 0
    assert sign(-0.5, 1e-9) == -1
    assert close(0.1 + 0.2, 0.3, 1e-12)
    assert not (0.1 + 0.2 == 0.3)

@test
fn sin_cos_matches_known_angles():
    let q = sin_cos(PI / 6)
    assert close(q.sin, 0.5, 1e-14) and close(q.cos, sqrt(3.0) / 2, 1e-14)
    let far = sin_cos(-100.0)
    assert close(far.sin, 0.5063656411097588, 1e-12) and close(far.cos, 0.8623188722876839, 1e-12)
    assert close(sin_cos(PI).cos, -1.0, 1e-14)
