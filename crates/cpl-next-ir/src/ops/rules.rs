//! The operations an engine and the C++ runtime both run, row by row. A row's guards (a
//! condition and the fault it raises) are written once, in Rust that the C++ emitter prints
//! as C++. Its kernel, what it computes once the guards pass, is written once per language,
//! side by side, and `cpp::tests::runtime_agrees_with_ops` runs the two against each other.
use super::{cpp_name, value, Fault};
use std::fmt::Write;

/// `Int`'s largest value, as a `U64`.
pub const INT_LAST: u64 = i64::MAX as u64;
/// The largest `x` whose `bit_ceil` is an `Int`: `2^62`.
pub const BIT_CEIL_LAST: i64 = 1 << 62;
/// `2^63`: a whole `F64` is an `Int` exactly when it is strictly between `-INT_SPAN` and this.
pub const INT_SPAN: f64 = 9223372036854775808.0;
/// `Int`'s and `U64`'s largest values, as `I128`s.
pub const WIDE_INT_LAST: i128 = i64::MAX as i128;
pub const WIDE_U64_LAST: i128 = u64::MAX as i128;
type Unit = ();

macro_rules! kernels {
    ($( $name:ident($($p:ident: $t:ident),*) -> $r:ident $body:block $cpp:literal )*) => {
        mod kernel {
            use super::*;
            $( #[inline(always)] pub(super) fn $name($($p: $t),*) -> Result<$r, Fault> $body )*
        }
        fn kernels_cpp(s: &mut String) {
            $( cpp_fn(s, concat!("k_", stringify!($name)), &[$((stringify!($p), stringify!($t))),*],
                stringify!($r), $cpp); )*
        }
    };
}

macro_rules! rows {
    ($( $(#[$m:meta])* $name:ident($($p:ident: $t:ident),*) -> $r:ident
        [$($c:expr => $f:ident),*] $k:ident; )*) => {
        $(
            $(#[$m])*
            #[inline(always)]
            pub fn $name($($p: $t),*) -> Result<$r, Fault> {
                $( if $c { return Err(Fault::$f); } )*
                kernel::$k($($p),*)
            }
        )*
        fn rows_cpp(s: &mut String) {
            $( {
                let mut body = String::new();
                $( let _ = write!(body, "if ({}) cpl_fail(CPL_{}); ", stringify!($c), cpp_name(Fault::$f)); )*
                let _ = write!(body, "return cpl_k_{}({});", stringify!($k), [$(stringify!($p)),*].join(", "));
                cpp_fn(s, stringify!($name), &[$((stringify!($p), stringify!($t))),*], stringify!($r), &body);
            } )*
        }
    };
}

kernels! {
    add(a: i64, b: i64) -> i64 { value(a.checked_add(b)) }
        "int64_t r; if (__builtin_add_overflow(a, b, &r) || r == INT64_MIN) cpl_fail(CPL_OVERFLOW); return r;"
    sub(a: i64, b: i64) -> i64 { value(a.checked_sub(b)) }
        "int64_t r; if (__builtin_sub_overflow(a, b, &r) || r == INT64_MIN) cpl_fail(CPL_OVERFLOW); return r;"
    mul(a: i64, b: i64) -> i64 { value(a.checked_mul(b)) }
        "int64_t r; if (__builtin_mul_overflow(a, b, &r) || r == INT64_MIN) cpl_fail(CPL_OVERFLOW); return r;"
    neg(a: i64) -> i64 { value(a.checked_neg()) }
        "if (a == INT64_MIN) cpl_fail(CPL_OVERFLOW); return -a;"
    abs(a: i64) -> i64 { value(a.checked_abs()) }
        "if (a == INT64_MIN) cpl_fail(CPL_OVERFLOW); return a < 0 ? -a : a;"
    div(a: i64, b: i64) -> i64 { value(a.checked_div_euclid(b)) }
        "if (a == INT64_MIN && b == -1) cpl_fail(CPL_OVERFLOW); int64_t q = a / b, r = a % b; if (r < 0) q += b > 0 ? -1 : 1; return q;"
    rem(a: i64, b: i64) -> i64 { value(a.checked_rem_euclid(b)) }
        "if (a == INT64_MIN && b == -1) cpl_fail(CPL_OVERFLOW); int64_t r = a % b; if (r < 0) r += b > 0 ? b : -b; return r;"
    pow(a: i64, b: i64) -> i64 {
        // Past `u32` only 0, 1 and -1 have a power that is a value.
        value(match u32::try_from(b) {
            Ok(e) => a.checked_pow(e),
            Err(_) => match a {
                0 | 1 => Some(a),
                -1 => Some(if b % 2 == 0 { 1 } else { -1 }),
                _ => None,
            },
        })
    }
        "int64_t r = 1; for (; b > 0; b >>= 1) { if (b & 1) r = cpl_k_mul(r, a); if (b > 1) a = cpl_k_mul(a, a); } return r;"
    shl(a: i64, b: i64) -> i64 { value(Some(a << b).filter(|s| *s >= 0 && s >> b == a)) }
        "if (a > (INT64_MAX >> b)) cpl_fail(CPL_OVERFLOW); return a << b;"
    shr(a: i64, b: i64) -> i64 { Ok(a >> b) } "return a >> b;"
    band(a: i64, b: i64) -> i64 { value(Some(a & b)) }
        "int64_t r = a & b; if (r == INT64_MIN) cpl_fail(CPL_OVERFLOW); return r;"
    bxor(a: i64, b: i64) -> i64 { value(Some(a ^ b)) }
        "int64_t r = a ^ b; if (r == INT64_MIN) cpl_fail(CPL_OVERFLOW); return r;"
    udiv(a: u64, b: u64) -> u64 { Ok(a / b) } "return a / b;"
    urem(a: u64, b: u64) -> u64 { Ok(a % b) } "return a % b;"
    upow(a: u64, b: u64) -> u64 {
        let (mut r, mut x, mut e) = (1u64, a, b);
        while e > 0 {
            if e & 1 == 1 {
                r = r.wrapping_mul(x);
            }
            x = x.wrapping_mul(x);
            e >>= 1;
        }
        Ok(r)
    }
        "uint64_t r = 1; for (; b > 0; b >>= 1) { if (b & 1) r *= a; a *= a; } return r;"
    ushl(a: u64, b: i64) -> u64 { Ok(a << b) } "return a << b;"
    ushr(a: u64, b: i64) -> u64 { Ok(a >> b) } "return a >> b;"
    add_wrap(a: i64, b: i64) -> i64 { Ok(a.wrapping_add(b)) } "return (int64_t)((uint64_t)a + (uint64_t)b);"
    sub_wrap(a: i64, b: i64) -> i64 { Ok(a.wrapping_sub(b)) } "return (int64_t)((uint64_t)a - (uint64_t)b);"
    mul_wrap(a: i64, b: i64) -> i64 { Ok(a.wrapping_mul(b)) } "return (int64_t)((uint64_t)a * (uint64_t)b);"
    pow_wrap(a: i64, b: i64) -> i64 { upow(a as u64, b as u64).map(|w| w as i64) }
        "return (int64_t)cpl_k_upow((uint64_t)a, (uint64_t)b);"
    shl_wrap(a: i64, b: i64) -> i64 { Ok(((a as u64) << b) as i64) } "return (int64_t)((uint64_t)a << b);"
    popcount(u: u64) -> i64 { Ok(i64::from(u.count_ones())) } "return __builtin_popcountll(u);"
    ctz(u: u64) -> i64 { Ok(i64::from(u.trailing_zeros())) } "return u ? __builtin_ctzll(u) : 64;"
    clz(u: u64) -> i64 { Ok(i64::from(u.leading_zeros())) } "return u ? __builtin_clzll(u) : 64;"
    bit_width(u: u64) -> i64 { clz(u).map(|z| 64 - z) } "return 64 - cpl_k_clz(u);"
    bit_ceil(a: i64) -> i64 { Ok(if a <= 1 { 1 } else { 1 << (64 - ((a - 1) as u64).leading_zeros()) }) }
        "return a <= 1 ? 1 : (int64_t)1 << (64 - __builtin_clzll((uint64_t)(a - 1)));"
    cell(i: i64, len: i64) -> usize { debug_assert!(i < len); Ok(i as usize) } "return (size_t)i;"
    int_of(u: u64) -> i64 { Ok(u as i64) } "return (int64_t)u;"
    char_of(x: i64) -> u8 { Ok(x as u8) } "return cpl_char{(uint8_t)x};"
    size(n: i64, cell: i64, memory: i64) -> usize { let _ = (cell, memory); Ok(n as usize) } "return (size_t)n;"
    pass(ok: bool) -> Unit { let _ = ok; Ok(()) } "return {};"
    // The `F64` fns are the C library's in both languages: Rust's `f64` methods call it too.
    sqrt(a: f64) -> f64 { Ok(a.sqrt()) } "return __builtin_sqrt(a);"
    sin(a: f64) -> f64 { Ok(a.sin()) } "return __builtin_sin(a);"
    cos(a: f64) -> f64 { Ok(a.cos()) } "return __builtin_cos(a);"
    tan(a: f64) -> f64 { Ok(a.tan()) } "return __builtin_tan(a);"
    asin(a: f64) -> f64 { Ok(a.asin()) } "return __builtin_asin(a);"
    acos(a: f64) -> f64 { Ok(a.acos()) } "return __builtin_acos(a);"
    atan2(a: f64, b: f64) -> f64 { Ok(a.atan2(b)) } "return __builtin_atan2(a, b);"
    hypot(a: f64, b: f64) -> f64 { Ok(a.hypot(b)) } "return __builtin_hypot(a, b);"
    exp(a: f64) -> f64 { Ok(a.exp()) } "return __builtin_exp(a);"
    ln(a: f64) -> f64 { Ok(a.ln()) } "return __builtin_log(a);"
    floor(a: f64) -> f64 { Ok(a.floor()) } "return __builtin_floor(a);"
    ceil(a: f64) -> f64 { Ok(a.ceil()) } "return __builtin_ceil(a);"
    round(a: f64) -> f64 { Ok(a.round()) } "return __builtin_round(a);"
    trunc(a: f64) -> f64 { Ok(a.trunc()) } "return __builtin_trunc(a);"
    whole(y: f64) -> i64 { Ok(y as i64) } "return (int64_t)y;"
    wadd(a: i128, b: i128) -> i128 { a.checked_add(b).ok_or(Fault::Overflow) }
        "__int128 r; if (__builtin_add_overflow(a, b, &r)) cpl_fail(CPL_OVERFLOW); return r;"
    wsub(a: i128, b: i128) -> i128 { a.checked_sub(b).ok_or(Fault::Overflow) }
        "__int128 r; if (__builtin_sub_overflow(a, b, &r)) cpl_fail(CPL_OVERFLOW); return r;"
    wmul(a: i128, b: i128) -> i128 { a.checked_mul(b).ok_or(Fault::Overflow) }
        "__int128 r; if (__builtin_mul_overflow(a, b, &r)) cpl_fail(CPL_OVERFLOW); return r;"
    wpow(a: i128, b: i128) -> i128 {
        // Past `u32` only 0, 1 and -1 have a power that is a value.
        match u32::try_from(b) {
            Ok(e) => a.checked_pow(e),
            Err(_) => match a {
                0 | 1 => Some(a),
                -1 => Some(if b % 2 == 0 { 1 } else { -1 }),
                _ => None,
            },
        }
        .ok_or(Fault::Overflow)
    }
        "__int128 r = 1; for (; b > 0; b >>= 1) { if (b & 1) r = cpl_k_wmul(r, a); if (b > 1) a = cpl_k_wmul(a, a); } return r;"
    wneg(a: i128) -> i128 { a.checked_neg().ok_or(Fault::Overflow) }
        "if (a == I128_MIN) cpl_fail(CPL_OVERFLOW); return -a;"
    wabs(a: i128) -> i128 { a.checked_abs().ok_or(Fault::Overflow) }
        "if (a == I128_MIN) cpl_fail(CPL_OVERFLOW); return a < 0 ? -a : a;"
    wdiv(a: i128, b: i128) -> i128 { a.checked_div_euclid(b).ok_or(Fault::Overflow) }
        "if (a == I128_MIN && b == -1) cpl_fail(CPL_OVERFLOW); __int128 q = a / b, r = a % b; if (r < 0) q += b > 0 ? -1 : 1; return q;"
    wrem(a: i128, b: i128) -> i128 { a.checked_rem_euclid(b).ok_or(Fault::Overflow) }
        "if (a == I128_MIN && b == -1) cpl_fail(CPL_OVERFLOW); __int128 r = a % b; if (r < 0) r += b > 0 ? b : -b; return r;"
    wshl(a: i128, b: i64) -> i128 { Some(a << b).filter(|s| *s >= 0 && s >> b == a).ok_or(Fault::Overflow) }
        "if (a > (I128_MAX >> b)) cpl_fail(CPL_OVERFLOW); return a << b;"
    wshr(a: i128, b: i64) -> i128 { Ok(a >> b) } "return a >> b;"
    wband(a: i128, b: i128) -> i128 { Ok(a & b) } "return a & b;"
    wbor(a: i128, b: i128) -> i128 { Ok(a | b) } "return a | b;"
    wbxor(a: i128, b: i128) -> i128 { Ok(a ^ b) } "return a ^ b;"
    wide_of(a: i64) -> i128 { Ok(i128::from(a)) } "return (__int128)a;"
    wide_of_u64(u: u64) -> i128 { Ok(i128::from(u)) } "return (__int128)u;"
    int_of_wide(a: i128) -> i64 { Ok(a as i64) } "return (int64_t)a;"
    u64_of_wide(a: i128) -> u64 { Ok(a as u64) } "return (uint64_t)a;"
}

rows! {
    add(a: i64, b: i64) -> i64 [] add;
    sub(a: i64, b: i64) -> i64 [] sub;
    mul(a: i64, b: i64) -> i64 [] mul;
    /// `-a` on an `Int`.
    neg(a: i64) -> i64 [] neg;
    /// `abs` of an `Int`.
    abs(a: i64) -> i64 [] abs;
    div(a: i64, b: i64) -> i64 [b == 0 => DivisionByZero] div;
    rem(a: i64, b: i64) -> i64 [b == 0 => DivisionByZero] rem;
    pow(a: i64, b: i64) -> i64 [b < 0 => NegativePower] pow;
    shl(a: i64, b: i64) -> i64 [(b >> 6) != 0 => ShiftOutside, a < 0 => ShlNegative] shl;
    shr(a: i64, b: i64) -> i64 [(b >> 6) != 0 => ShiftOutside] shr;
    band(a: i64, b: i64) -> i64 [] band;
    bxor(a: i64, b: i64) -> i64 [] bxor;
    udiv(a: u64, b: u64) -> u64 [b == 0 => DivisionByZero] udiv;
    urem(a: u64, b: u64) -> u64 [b == 0 => DivisionByZero] urem;
    upow(a: u64, b: u64) -> u64 [] upow;
    // A shift's amount is an `Int`, in 0..64 exactly when its bits past the sixth are zero,
    // and an ASCII code is one in 0..128.
    ushl(a: u64, b: i64) -> u64 [(b >> 6) != 0 => ShiftOutside] ushl;
    ushr(a: u64, b: i64) -> u64 [(b >> 6) != 0 => ShiftOutside] ushr;
    // `Int` arithmetic where overflow checks are off: it wraps to 64 bits (R09.4.2).
    add_wrap(a: i64, b: i64) -> i64 [] add_wrap;
    sub_wrap(a: i64, b: i64) -> i64 [] sub_wrap;
    mul_wrap(a: i64, b: i64) -> i64 [] mul_wrap;
    pow_wrap(a: i64, b: i64) -> i64 [b < 0 => NegativePower] pow_wrap;
    shl_wrap(a: i64, b: i64) -> i64 [(b >> 6) != 0 => ShiftOutside, a < 0 => ShlNegative] shl_wrap;
    popcount(u: u64) -> i64 [] popcount;
    ctz(u: u64) -> i64 [] ctz;
    clz(u: u64) -> i64 [] clz;
    bit_width(u: u64) -> i64 [] bit_width;
    /// The smallest power of two at least `a`, 1 for `a <= 1`.
    bit_ceil(a: i64) -> i64 [a > BIT_CEIL_LAST => Overflow] bit_ceil;
    /// A `U64` as an `Int`.
    int_of_u64(u: u64) -> i64 [u > INT_LAST => U64PastInt] int_of;
    /// The character with code `x`, an ASCII one.
    char_of(x: i64) -> u8 [(x >> 7) != 0 => CharPastAscii] char_of;
    /// The number of cells `raw_new(n, _)` makes, each of `cell` bytes, under `memory` bytes.
    storage_size(n: i64, cell: i64, memory: i64) -> usize [n < 0 => NegativeSize, n > memory / cell => PastMemoryLimit] size;
    /// Cell `i` of storage with `len` cells, for a read and for a write.
    read_cell(i: i64, len: i64) -> usize [(i < 0) | (i >= len) => ReadOutside] cell;
    write_cell(i: i64, len: i64) -> usize [(i < 0) | (i >= len) => WriteOutside] cell;
    // The checks std makes (`key_check`, `range_check`, `token_check`, `input_check`): each passes when its operand is true.
    check_key(ok: bool) -> Unit [!ok => KeyOutside] pass;
    check_range(ok: bool) -> Unit [!ok => RangeCut] pass;
    check_variant(ok: bool) -> Unit [!ok => NoVariant] pass;
    check_token(ok: bool) -> Unit [!ok => NoToken] pass;
    check_input(ok: bool) -> Unit [!ok => InputOutside] pass;
    sqrt(a: f64) -> f64 [] sqrt;
    sin(a: f64) -> f64 [] sin;
    cos(a: f64) -> f64 [] cos;
    tan(a: f64) -> f64 [] tan;
    asin(a: f64) -> f64 [] asin;
    acos(a: f64) -> f64 [] acos;
    atan2(a: f64, b: f64) -> f64 [] atan2;
    hypot(a: f64, b: f64) -> f64 [] hypot;
    exp(a: f64) -> f64 [] exp;
    ln(a: f64) -> f64 [] ln;
    /// `floor`, `ceil`, `round` (halves away from zero) and `trunc`, still an `F64`.
    floor(a: f64) -> f64 [] floor;
    ceil(a: f64) -> f64 [] ceil;
    round(a: f64) -> f64 [] round;
    trunc(a: f64) -> f64 [] trunc;
    /// A whole `F64` as an `Int`; NaN and anything past `Int` trap.
    int_of_f64(y: f64) -> i64 [!(y > -INT_SPAN && y < INT_SPAN) => F64PastInt] whole;
    // `I128` arithmetic: it traps where its value is past 128 bits (R02.1); a shift's amount
    // is an `Int` in 0..128.
    wadd(a: i128, b: i128) -> i128 [] wadd;
    wsub(a: i128, b: i128) -> i128 [] wsub;
    wmul(a: i128, b: i128) -> i128 [] wmul;
    wpow(a: i128, b: i128) -> i128 [b < 0 => NegativePower] wpow;
    wneg(a: i128) -> i128 [] wneg;
    wabs(a: i128) -> i128 [] wabs;
    wdiv(a: i128, b: i128) -> i128 [b == 0 => DivisionByZero] wdiv;
    wrem(a: i128, b: i128) -> i128 [b == 0 => DivisionByZero] wrem;
    wshl(a: i128, b: i64) -> i128 [(b >> 7) != 0 => ShiftOutside, a < 0 => ShlNegative] wshl;
    wshr(a: i128, b: i64) -> i128 [(b >> 7) != 0 => ShiftOutside] wshr;
    wband(a: i128, b: i128) -> i128 [] wband;
    wbor(a: i128, b: i128) -> i128 [] wbor;
    wbxor(a: i128, b: i128) -> i128 [] wbxor;
    /// An `Int` or a `U64` as an `I128`.
    i128_of(a: i64) -> i128 [] wide_of;
    i128_of_u64(u: u64) -> i128 [] wide_of_u64;
    /// An `I128` as an `Int`, and as a `U64`.
    int_of_i128(a: i128) -> i64 [a > WIDE_INT_LAST => U64PastInt, a < -WIDE_INT_LAST => U64PastInt] int_of_wide;
    u64_of_i128(a: i128) -> u64 [a < 0 => Overflow, a > WIDE_U64_LAST => Overflow] u64_of_wide;
}

fn cpp_fn(s: &mut String, name: &str, params: &[(&str, &str)], ret: &str, body: &str) {
    let ps: Vec<String> = params
        .iter()
        .map(|(p, t)| format!("{} {p}", cty(t)))
        .collect();
    let _ = writeln!(
        s,
        "static inline {} cpl_{name}({}) {{ {body} }}",
        cty(ret),
        ps.join(", ")
    );
}

/// Each Rust type a row is written in, with its C++ type.
const CTYS: [(&str, &str); 8] = [
    ("i64", "int64_t"),
    ("f64", "double"),
    ("i128", "__int128"),
    ("u64", "uint64_t"),
    ("u8", "cpl_char"),
    ("usize", "size_t"),
    ("bool", "bool"),
    ("Unit", "cpl_unit"),
];

/// The C++ type of a row's Rust type.
fn cty(t: &str) -> &'static str {
    CTYS.iter()
        .find(|(r, _)| *r == t)
        .map_or("void", |(_, c)| c)
}

/// Every row as a C++ helper (`cpl_div`), after its kernel (`cpl_k_div`).
pub fn cpp() -> String {
    let mut s = format!(
        "static const uint64_t INT_LAST = {INT_LAST}ULL;\n\
         static const int64_t BIT_CEIL_LAST = {BIT_CEIL_LAST}LL;\n\
         static const double INT_SPAN = {INT_SPAN:?};\n\
         static const __int128 WIDE_INT_LAST = (__int128)INT64_MAX;\n\
         static const __int128 WIDE_U64_LAST = (__int128)UINT64_MAX;\n\
         static const __int128 I128_MAX = (__int128)(((unsigned __int128)1 << 127) - 1);\n\
         static const __int128 I128_MIN = -I128_MAX - 1;\n"
    );
    kernels_cpp(&mut s);
    rows_cpp(&mut s);
    s
}
