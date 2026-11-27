//! The meaning of every operation the IR names: what it computes and how it fails. The
//! reference interpreter and the engines call these functions, and the C++ emitter takes
//! its helper names and its trap table from here, so no operation is written twice.
use crate::ir::{CheckKind, Intr};
use cpl_diag::{codes, Code};

mod rules;
pub use rules::{abs, char_of, cpp as cpp_runtime, int_of_u64, neg, storage_size};

/// Every way a run can fail, one row each, with the code it carries and its message.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Fault {
    Overflow,
    U64PastInt,
    CharPastAscii,
    DivisionByZero,
    NegativePower,
    ShiftOutside,
    ShlNegative,
    NegativeSize,
    PastMemoryLimit,
    ReadOutside,
    WriteOutside,
    KeyOutside,
    NoVariant,
    RangeCut,
    BadIntToken,
    IntPastInt,
    StackExhausted,
    /// A `Check` statement: its code comes from its kind, its message from the IR.
    Check(CheckKind),
    /// The IR broke a rule the layers before the backend keep. Never the program's fault.
    Internal(&'static str),
}

/// The faults an operation raises by itself, in the order of the C++ runtime's trap table.
pub const FAULTS: [Fault; 17] = [
    Fault::Overflow,
    Fault::U64PastInt,
    Fault::CharPastAscii,
    Fault::DivisionByZero,
    Fault::NegativePower,
    Fault::ShiftOutside,
    Fault::ShlNegative,
    Fault::NegativeSize,
    Fault::PastMemoryLimit,
    Fault::ReadOutside,
    Fault::WriteOutside,
    Fault::KeyOutside,
    Fault::NoVariant,
    Fault::RangeCut,
    Fault::BadIntToken,
    Fault::IntPastInt,
    Fault::StackExhausted,
];

/// The largest storage one `raw_new` may make: a stand-in for the run's memory limit
/// (contract 0, `Limits`).
pub const MAX_STORAGE: i64 = 1 << 28;

/// The code a failure is reported with when the fault is in the compiler.
pub const INTERNAL: &str = "internal";

impl Fault {
    pub fn code(self) -> &'static str {
        let c: Code = match self {
            Fault::Overflow | Fault::U64PastInt => codes::OVERFLOW,
            Fault::CharPastAscii => codes::RANGE_STORE,
            Fault::DivisionByZero => codes::DIVISION_BY_ZERO,
            Fault::NegativePower
            | Fault::ShiftOutside
            | Fault::ShlNegative
            | Fault::NegativeSize
            | Fault::PastMemoryLimit
            | Fault::RangeCut => codes::RUNTIME,
            Fault::ReadOutside | Fault::WriteOutside | Fault::KeyOutside => {
                codes::INDEX_OUT_OF_RANGE
            }
            Fault::NoVariant => codes::DOMAIN_CAST,
            Fault::BadIntToken => codes::BAD_INPUT_TOKEN,
            Fault::IntPastInt => codes::INPUT_OUT_OF_CONTRACT,
            Fault::StackExhausted => codes::STACK_EXHAUSTED,
            Fault::Check(k) => k.code(),
            Fault::Internal(_) => return INTERNAL,
        };
        c.0
    }
    /// The message; a `Check`'s is the one its statement carries.
    pub fn message(self) -> &'static str {
        match self {
            Fault::Overflow => "integer overflow",
            Fault::U64PastInt => "the value does not fit in an Int",
            Fault::CharPastAscii => "a `Char` is 0..=127",
            Fault::DivisionByZero => "division by zero",
            Fault::NegativePower => "a negative power",
            Fault::ShiftOutside => "a shift outside 0..64",
            Fault::ShlNegative => "`<<` of a negative number",
            Fault::NegativeSize => "negative storage size",
            Fault::PastMemoryLimit => "storage larger than the memory limit",
            Fault::ReadOutside => "read outside storage",
            Fault::WriteOutside => "write outside storage",
            Fault::KeyOutside => "key outside the sequence",
            Fault::NoVariant => "no variant has this value",
            Fault::RangeCut => "a cut of a range outside it",
            Fault::BadIntToken => "expected an integer in the input",
            Fault::IntPastInt => "a whole number in the input does not fit in an `Int`",
            Fault::StackExhausted => "the stack ran out",
            Fault::Check(_) => "a check failed",
            Fault::Internal(rule) => rule,
        }
    }
    /// Its row in the C++ runtime's trap table.
    pub fn row(self) -> usize {
        FAULTS.iter().position(|f| *f == self).unwrap_or(0)
    }
}

/// `Int` is `-(2^63-1)..=2^63-1`: the lowest bit pattern is not a value.
fn value(r: Option<i64>) -> Result<i64, Fault> {
    match r {
        Some(r) if r != i64::MIN => Ok(r),
        _ => Err(Fault::Overflow),
    }
}

/// A binary operation on two `Int`s (R03.3).
pub fn int(op: Intr, a: i64, b: i64) -> Result<i64, Fault> {
    use rules as r;
    match op {
        Intr::Add => r::add(a, b),
        Intr::Sub => r::sub(a, b),
        Intr::Mul => r::mul(a, b),
        Intr::Div => r::div(a, b),
        Intr::Rem => r::rem(a, b),
        Intr::Pow => r::pow(a, b),
        Intr::Shl => r::shl(a, b),
        Intr::Shr => r::shr(a, b),
        Intr::BitAnd => r::band(a, b),
        // `a | b` of two values is never the lowest bit pattern.
        Intr::BitOr => Ok(a | b),
        Intr::BitXor => r::bxor(a, b),
        Intr::AddWrap => r::add_wrap(a, b),
        Intr::SubWrap => r::sub_wrap(a, b),
        Intr::MulWrap => r::mul_wrap(a, b),
        Intr::PowWrap => r::pow_wrap(a, b),
        Intr::ShlWrap => r::shl_wrap(a, b),
        _ => Err(Fault::Internal("not an Int operation")),
    }
}

/// A binary operation on `U64`s: it wraps; division by zero and a shift outside 0..64 trap
/// (R03.4.1). A shift's amount is an `Int`, passed as its bits.
pub fn word(op: Intr, a: u64, b: u64) -> Result<u64, Fault> {
    use rules as r;
    Ok(match op {
        Intr::Add => a.wrapping_add(b),
        Intr::Sub => a.wrapping_sub(b),
        Intr::Mul => a.wrapping_mul(b),
        Intr::Div => return r::udiv(a, b),
        Intr::Rem => return r::urem(a, b),
        Intr::Pow => return r::upow(a, b),
        Intr::Shl => return r::ushl(a, b as i64),
        Intr::Shr => return r::ushr(a, b as i64),
        Intr::BitAnd => a & b,
        Intr::BitOr => a | b,
        Intr::BitXor => a ^ b,
        _ => return Err(Fault::Internal("not a U64 operation")),
    })
}

/// A comparison. Every comparable value is an integer of at most 64 bits, signed or not.
pub fn compare(op: Intr, a: i128, b: i128) -> Result<bool, Fault> {
    Ok(match op {
        Intr::Eq => a == b,
        Intr::Ne => a != b,
        Intr::Lt => a < b,
        Intr::Le => a <= b,
        Intr::Gt => a > b,
        Intr::Ge => a >= b,
        Intr::Below => 0 <= a && a < b,
        _ => return Err(Fault::Internal("not a comparison")),
    })
}

/// A binary operation on `F64`s, IEEE (R02.1).
pub fn float(op: Intr, a: f64, b: f64) -> Result<f64, Fault> {
    Ok(match op {
        Intr::Add => a + b,
        Intr::Sub => a - b,
        Intr::Mul => a * b,
        Intr::Div => a / b,
        _ => return Err(Fault::Internal("not an F64 operation")),
    })
}

/// `-a` on an `F64`.
pub fn neg_float(a: f64) -> f64 {
    -a
}

/// The `F64` nearest an `Int`, ties to even.
pub fn f64_of_int(i: i64) -> f64 {
    i as f64
}

/// `abs` of an F64: the value with its sign cleared, NaN included.
pub fn abs_f64(x: f64) -> f64 {
    x.abs()
}

/// A comparison of `F64`s: `==` is exact (R02.1).
pub fn compare_float(op: Intr, a: f64, b: f64) -> Result<bool, Fault> {
    Ok(match op {
        Intr::Eq => a == b,
        Intr::Ne => a != b,
        Intr::Lt => a < b,
        Intr::Le => a <= b,
        Intr::Gt => a > b,
        Intr::Ge => a >= b,
        _ => return Err(Fault::Internal("not an F64 comparison")),
    })
}

/// An `Int`'s bits as a `U64`.
pub fn u64_of_int(i: i64) -> u64 {
    i as u64
}

/// `popcount`, `ctz`, `clz`, `bit_width` or `bit_ceil` of a 64-bit word, an `Int` as its bits.
pub fn bit_count(op: Intr, w: u64) -> Result<i64, Fault> {
    match op {
        Intr::Popcount => rules::popcount(w),
        Intr::Ctz => rules::ctz(w),
        Intr::Clz => rules::clz(w),
        Intr::BitWidth => rules::bit_width(w),
        Intr::BitCeil => rules::bit_ceil(w as i64),
        _ => Err(Fault::Internal("not a bit count")),
    }
}

pub fn not(b: bool) -> bool {
    !b
}

/// A `Char`'s code.
pub fn code_of(c: u8) -> i64 {
    i64::from(c)
}

/// Whitespace to a token read: space, `\t`, `\n`, `\v`, `\f` and `\r` (R10.2.8).
pub fn is_space(c: u8) -> bool {
    c == b' ' || (b'\t'..=b'\r').contains(&c)
}

/// The input's tokens, split at runs of whitespace (R10.2.8).
pub fn tokens(input: &str) -> impl Iterator<Item = &str> {
    input
        .split(|c: char| u8::try_from(c).is_ok_and(is_space))
        .filter(|t| !t.is_empty())
}

/// The `Int` an input token spells. A whole number past `Int` is out of the contract, not a
/// bad token (R10.2.6); the end of the input is a bad token.
pub fn read_int(t: Option<&str>) -> Result<i64, Fault> {
    let t = t.unwrap_or("");
    let digits = t.strip_prefix('-').unwrap_or(t);
    match t.parse::<i64>().ok().filter(|v| *v != i64::MIN) {
        Some(i) => Ok(i),
        None if !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()) => {
            Err(Fault::IntPastInt)
        }
        None => Err(Fault::BadIntToken),
    }
}

/// A value `print` writes and an event shows as text.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Scalar {
    Int(i64),
    U64(u64),
    /// An `F64`, by its bits.
    F64(u64),
    Bool(bool),
    Char(u8),
}
/// `v` as `print` writes it.
pub fn show(v: Scalar, out: &mut String) {
    show_fmt(v, Fmt::default(), out)
}

/// How `print` lays out a number: `"{x:6}"`, `"{x:04}"`, `"{x:.9}"` or `precision:` (R10.3.2).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Fmt {
    /// The least width, filled on the left.
    pub width: u32,
    /// Filled with zeros after the sign, not spaces.
    pub zero: bool,
    /// Digits after an `F64`'s point.
    pub digits: Option<u32>,
}

/// `v` as text, laid out by `f`.
pub fn show_fmt(v: Scalar, f: Fmt, out: &mut String) {
    use std::fmt::Write;
    let w = f.width as usize;
    let _ = match v {
        Scalar::Int(i) if f.zero => write!(out, "{i:0w$}"),
        Scalar::Int(i) => write!(out, "{i:w$}"),
        Scalar::U64(u) if f.zero => write!(out, "{u:0w$}"),
        Scalar::U64(u) => write!(out, "{u:w$}"),
        // Ten decimals unless a precision is given (R10.3.4).
        // Not a number and the infinities are spelled as C's printf spells them, and padded
        // with spaces only, so a C++ build prints what the engine prints.
        Scalar::F64(b) if !f64::from_bits(b).is_finite() => {
            let x = f64::from_bits(b);
            let sign = if x.is_sign_negative() { "-" } else { "" };
            let name = if x.is_nan() { "nan" } else { "inf" };
            write!(out, "{:>w$}", format!("{sign}{name}"))
        }
        Scalar::F64(b) => {
            let (x, d) = (f64::from_bits(b), f.digits.unwrap_or(10) as usize);
            match f.zero {
                true => write!(out, "{x:0w$.d$}"),
                false => write!(out, "{x:w$.d$}"),
            }
        }
        Scalar::Bool(b) => write!(out, "{b}"),
        Scalar::Char(c) => write!(out, "{}", char::from(c)),
    };
}

/// Cell `i` of storage of `len` cells, for a read or a write.
pub fn cell(i: i64, len: usize, write: bool) -> Result<usize, Fault> {
    let len = len as i64;
    if write {
        rules::write_cell(i, len)
    } else {
        rules::read_cell(i, len)
    }
}

/// A check std makes (`key_check`, `range_check`): passes when its operand is true.
pub fn check(op: Intr, ok: bool) -> Result<(), Fault> {
    match op {
        Intr::CheckKey => rules::check_key(ok),
        Intr::CheckRange => rules::check_range(ok),
        Intr::CheckVariant => rules::check_variant(ok),
        _ => Err(Fault::Internal("not a check")),
    }
}

/// How the C++ writes an operation: a runtime helper, or a C++ operator that cannot fail.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cpp {
    Call(&'static str),
    Infix(&'static str),
}

/// The C++ for an arithmetic or comparison operation; `unsigned` for `U64` operands.
pub fn cpp(op: Intr, unsigned: bool) -> Option<Cpp> {
    use Cpp::{Call, Infix};
    Some(match (op, unsigned) {
        (Intr::Add, true) => Infix("+"),
        (Intr::Sub, true) => Infix("-"),
        (Intr::Mul, true) => Infix("*"),
        (Intr::Div, true) => Call("cpl_udiv"),
        (Intr::Rem, true) => Call("cpl_urem"),
        (Intr::Pow, true) => Call("cpl_upow"),
        (Intr::Shl, true) => Call("cpl_ushl"),
        (Intr::Shr, true) => Call("cpl_ushr"),
        (Intr::BitAnd, true) => Infix("&"),
        (Intr::BitXor, true) => Infix("^"),
        (Intr::Add, false) => Call("cpl_add"),
        (Intr::Sub, false) => Call("cpl_sub"),
        (Intr::Mul, false) => Call("cpl_mul"),
        (Intr::Div, false) => Call("cpl_div"),
        (Intr::Rem, false) => Call("cpl_rem"),
        (Intr::Pow, false) => Call("cpl_pow"),
        (Intr::Shl, false) => Call("cpl_shl"),
        (Intr::Shr, false) => Call("cpl_shr"),
        (Intr::BitAnd, false) => Call("cpl_band"),
        (Intr::BitXor, false) => Call("cpl_bxor"),
        (Intr::AddWrap, _) => Call("cpl_add_wrap"),
        (Intr::SubWrap, _) => Call("cpl_sub_wrap"),
        (Intr::MulWrap, _) => Call("cpl_mul_wrap"),
        (Intr::PowWrap, _) => Call("cpl_pow_wrap"),
        (Intr::ShlWrap, _) => Call("cpl_shl_wrap"),
        // `a | b` of two values is never the lowest bit pattern.
        (Intr::BitOr, _) => Infix("|"),
        (Intr::Eq, _) => Infix("=="),
        (Intr::Ne, _) => Infix("!="),
        (Intr::Lt, _) => Infix("<"),
        (Intr::Le, _) => Infix("<="),
        (Intr::Gt, _) => Infix(">"),
        (Intr::Ge, _) => Infix(">="),
        _ => return None,
    })
}

/// The C++ runtime's trap table, one `{code, message}` row per entry of `FAULTS`, and a
/// name per row (`CPL_OVERFLOW`) for the helpers to trap with.
pub fn cpp_trap_table() -> String {
    let mut s = String::from("static const char* const cpl_faults[][2] = {\n");
    for f in FAULTS {
        s.push_str(&format!("    {{{:?}, {:?}}},\n", f.code(), f.message()));
    }
    s.push_str("};\n");
    for (i, f) in FAULTS.iter().enumerate() {
        s.push_str(&format!("#define CPL_{} {i}\n", cpp_name(*f)));
    }
    s
}

/// `Fault::DivisionByZero` is `DIVISION_BY_ZERO` in the C++.
fn cpp_name(f: Fault) -> String {
    let mut out = String::new();
    for (i, c) in format!("{f:?}").chars().enumerate() {
        if c.is_ascii_uppercase() && i > 0 {
            out.push('_');
        }
        out.push(c.to_ascii_uppercase());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_token_read_splits_at_ascii_whitespace_only() {
        let got: Vec<&str> = tokens(" 1\t2\n\x0b3\x0c\r\n4\u{a0}5\u{85}6 ").collect();
        assert_eq!(got, ["1", "2", "3", "4\u{a0}5\u{85}6"]);
    }
}
