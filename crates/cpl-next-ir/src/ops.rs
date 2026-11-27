//! The meaning of every operation the IR names: what it computes and how it fails. The
//! reference interpreter and the engines call these functions, and the C++ emitter takes
//! its helper names and its trap table from here, so no operation is written twice.
use crate::ir::{CheckKind, Intr};
use cpl_diag::{codes, Code};

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
    RangeCut,
    BadIntToken,
    StackExhausted,
    /// A `Check` statement: its code comes from its kind, its message from the IR.
    Check(CheckKind),
    /// The IR broke a rule the layers before the backend keep. Never the program's fault.
    Internal(&'static str),
}

/// The faults an operation raises by itself, in the order of the C++ runtime's trap table.
pub const FAULTS: [Fault; 15] = [
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
    Fault::RangeCut,
    Fault::BadIntToken,
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
            Fault::BadIntToken => codes::BAD_INPUT_TOKEN,
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
            Fault::RangeCut => "a cut of a range outside it",
            Fault::BadIntToken => "expected an integer in the input",
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
    value(match op {
        Intr::Add => a.checked_add(b),
        Intr::Sub => a.checked_sub(b),
        Intr::Mul => a.checked_mul(b),
        Intr::Div | Intr::Rem if b == 0 => return Err(Fault::DivisionByZero),
        Intr::Div => a.checked_div_euclid(b),
        Intr::Rem => a.checked_rem_euclid(b),
        Intr::Pow if b < 0 => return Err(Fault::NegativePower),
        // Past `u32` only 0, 1 and -1 have a power that is a value.
        Intr::Pow => match u32::try_from(b) {
            Ok(e) => a.checked_pow(e),
            Err(_) => match a {
                0 | 1 => Some(a),
                -1 => Some(if b % 2 == 0 { 1 } else { -1 }),
                _ => None,
            },
        },
        Intr::Shl | Intr::Shr if !(0..64).contains(&b) => return Err(Fault::ShiftOutside),
        Intr::Shl if a < 0 => return Err(Fault::ShlNegative),
        Intr::Shl => Some(a << b).filter(|s| *s >= 0 && s >> b == a),
        Intr::Shr => Some(a >> b),
        Intr::BitAnd => Some(a & b),
        Intr::BitOr => Some(a | b),
        Intr::BitXor => Some(a ^ b),
        _ => return Err(Fault::Internal("not an Int operation")),
    })
}

pub fn neg(a: i64) -> Result<i64, Fault> {
    value(a.checked_neg())
}

/// A binary operation on `U64`s: it wraps; division by zero and a shift outside 0..64 trap
/// (R03.4.1). A shift's amount is an `Int`, passed as its bits.
pub fn word(op: Intr, a: u64, b: u64) -> Result<u64, Fault> {
    if matches!(op, Intr::Shl | Intr::Shr) && !(0..64).contains(&(b as i64)) {
        return Err(Fault::ShiftOutside);
    }
    Ok(match op {
        Intr::Add => a.wrapping_add(b),
        Intr::Sub => a.wrapping_sub(b),
        Intr::Mul => a.wrapping_mul(b),
        Intr::Div | Intr::Rem if b == 0 => return Err(Fault::DivisionByZero),
        Intr::Div => a / b,
        Intr::Rem => a % b,
        Intr::Pow => {
            let (mut r, mut x, mut e) = (1u64, a, b);
            while e > 0 {
                if e & 1 == 1 {
                    r = r.wrapping_mul(x);
                }
                x = x.wrapping_mul(x);
                e >>= 1;
            }
            r
        }
        Intr::Shl => a << b,
        Intr::Shr => a >> b,
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

pub fn int_of_u64(u: u64) -> Result<i64, Fault> {
    i64::try_from(u).map_err(|_| Fault::U64PastInt)
}

/// The character with code `x`, an ASCII one.
pub fn char_of(x: i64) -> Result<u8, Fault> {
    u8::try_from(x)
        .ok()
        .filter(|c| *c <= 127)
        .ok_or(Fault::CharPastAscii)
}

/// The number of cells `raw_new(n, _)` makes.
pub fn storage_size(n: i64) -> Result<usize, Fault> {
    match n {
        n if n < 0 => Err(Fault::NegativeSize),
        n if n > MAX_STORAGE => Err(Fault::PastMemoryLimit),
        n => Ok(n as usize),
    }
}

/// Cell `i` of storage of `len` cells, for a read or a write.
pub fn cell(i: i64, len: usize, write: bool) -> Result<usize, Fault> {
    match usize::try_from(i) {
        Ok(i) if i < len => Ok(i),
        _ if write => Err(Fault::WriteOutside),
        _ => Err(Fault::ReadOutside),
    }
}

/// A check std makes (`key_check`, `range_check`): passes when its operand is true.
pub fn check(op: Intr, ok: bool) -> Result<(), Fault> {
    match (op, ok) {
        (_, true) => Ok(()),
        (Intr::CheckKey, false) => Err(Fault::KeyOutside),
        (Intr::CheckRange, false) => Err(Fault::RangeCut),
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
