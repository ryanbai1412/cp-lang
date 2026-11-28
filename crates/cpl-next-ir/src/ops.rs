//! The meaning of every operation the IR names: what it computes and how it fails. The
//! reference interpreter and the engines call these functions, and the C++ emitter takes
//! its helper names and its trap table from here, so no operation is written twice.
//!
//! An operation a std fn names with `@fast(op)` is only a faster way to compute that fn's CP
//! body, which is what the fn means: `fast_ops_agree_with_their_bodies` runs both on every
//! runner and they must agree.
use crate::ir::{Arg, CheckKind, FloatFn, Intr, Rounding, EK};
use crate::prog::CTy;
use crate::run::Written;
use cpl_diag::{codes, Code};
use cpl_prim::PrimOp;

mod rules;
pub use rules::{
    abs, char_of, cpp as cpp_runtime, i128_of, i128_of_u64, int_of_i128, int_of_u64, neg,
    storage_size, u64_of_i128, wabs, wneg,
};

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
    NoToken,
    InputOutside,
    IntPastInt,
    StackExhausted,
    F64PastInt,
    /// A `Check` statement: its code comes from its kind, its message from the IR.
    Check(CheckKind),
    /// The IR broke a rule the layers before the backend keep. Never the program's fault.
    Internal(&'static str),
}

/// The faults an operation raises by itself, in the order of the C++ runtime's trap table.
pub const FAULTS: [Fault; 20] = [
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
    Fault::NoToken,
    Fault::InputOutside,
    Fault::IntPastInt,
    Fault::StackExhausted,
    Fault::F64PastInt,
];

/// The code a failure is reported with when the fault is in the compiler.
pub const INTERNAL: &str = "internal";

impl Fault {
    pub fn code(self) -> &'static str {
        let c: Code = match self {
            Fault::Overflow | Fault::U64PastInt | Fault::F64PastInt => codes::OVERFLOW,
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
            Fault::BadIntToken | Fault::NoToken => codes::BAD_INPUT_TOKEN,
            Fault::IntPastInt | Fault::InputOutside => codes::INPUT_OUT_OF_CONTRACT,
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
            Fault::NoToken => "expected a token in the input",
            Fault::InputOutside => "a value in the input is outside its contract",
            Fault::IntPastInt => "a whole number in the input does not fit in an `Int`",
            Fault::StackExhausted => "the stack ran out",
            Fault::F64PastInt => "the `F64` rounds to a value that does not fit in an `Int`",
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

/// A binary operation on `I128`s, which traps past 128 bits (R02.1). A shift's amount is an
/// `Int`.
pub fn wide(op: Intr, a: i128, b: i128) -> Result<i128, Fault> {
    use rules as r;
    let shift = || i64::try_from(b).unwrap_or(i64::MAX);
    match op {
        Intr::Add => r::wadd(a, b),
        Intr::Sub => r::wsub(a, b),
        Intr::Mul => r::wmul(a, b),
        Intr::Div => r::wdiv(a, b),
        Intr::Rem => r::wrem(a, b),
        Intr::Shl => r::wshl(a, shift()),
        Intr::Shr => r::wshr(a, shift()),
        Intr::BitAnd => r::wband(a, b),
        Intr::BitOr => r::wbor(a, b),
        Intr::BitXor => r::wbxor(a, b),
        _ => Err(Fault::Internal("not an I128 operation")),
    }
}

/// The C++ helper for an `I128` operation; comparisons are C++'s own.
pub fn cpp_wide(op: Intr) -> Option<Cpp> {
    use Cpp::{Call, Infix};
    Some(match op {
        Intr::Add => Call("cpl_wadd"),
        Intr::Sub => Call("cpl_wsub"),
        Intr::Mul => Call("cpl_wmul"),
        Intr::Div => Call("cpl_wdiv"),
        Intr::Rem => Call("cpl_wrem"),
        Intr::Shl => Call("cpl_wshl"),
        Intr::Shr => Call("cpl_wshr"),
        Intr::BitAnd => Call("cpl_wband"),
        Intr::BitOr => Call("cpl_wbor"),
        Intr::BitXor => Call("cpl_wbxor"),
        Intr::Neg => Call("cpl_wneg"),
        Intr::Abs => Call("cpl_wabs"),
        Intr::Eq => Infix("=="),
        Intr::Ne => Infix("!="),
        Intr::Lt => Infix("<"),
        Intr::Le => Infix("<="),
        Intr::Gt => Infix(">"),
        Intr::Ge => Infix(">="),
        Intr::ToI128 => Call("cpl_i128_of"),
        Intr::I128OfU64 => Call("cpl_i128_of_u64"),
        Intr::FromI128 => Call("cpl_int_of_i128"),
        Intr::F64OfI128 => Call("(double)"),
        Intr::U64OfI128 => Call("cpl_u64_of_i128"),
        _ => return None,
    })
}

/// A comparison. Every comparable value is an integer of at most 128 bits, signed or not.
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
        Intr::Float(f) => return float_fn(f, a, b),
        _ => return Err(Fault::Internal("not an F64 operation")),
    })
}

/// `sqrt`, `sin`, `atan2` and the rest: `b` is read only by the fns of two operands.
pub fn float_fn(f: FloatFn, a: f64, b: f64) -> Result<f64, Fault> {
    use rules as r;
    match f {
        FloatFn::Sqrt => r::sqrt(a),
        FloatFn::Sin => r::sin(a),
        FloatFn::Cos => r::cos(a),
        FloatFn::Tan => r::tan(a),
        FloatFn::Asin => r::asin(a),
        FloatFn::Acos => r::acos(a),
        FloatFn::Atan2 => r::atan2(a, b),
        FloatFn::Hypot => r::hypot(a, b),
        FloatFn::Exp => r::exp(a),
        FloatFn::Ln => r::ln(a),
    }
}

/// An `F64` rounded to an `Int`; NaN and anything past `Int` trap.
pub fn to_int(r: Rounding, x: f64) -> Result<i64, Fault> {
    rules::int_of_f64(match r {
        Rounding::Floor => rules::floor(x),
        Rounding::Ceil => rules::ceil(x),
        Rounding::Round => rules::round(x),
        Rounding::Trunc => rules::trunc(x),
    }?)
}

/// `-a` on an `F64`.
pub fn neg_float(a: f64) -> f64 {
    -a
}

/// The `F64` nearest an `Int`, ties to even.
pub fn f64_of_int(i: i64) -> f64 {
    i as f64
}

/// The `F64` nearest an `I128`, ties to even.
pub fn f64_of_i128(a: i128) -> f64 {
    a as f64
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

/// The program's input; token reads and line reads take from it in turn (R10.2.3). A read
/// that must wait for more of a stream flushes the run's output first (R10.4). The C++
/// runtime's `cpl_has`, `cpl_read` and `cpl_read_byte` keep the same rules.
pub struct Input {
    bytes: Vec<u8>,
    at: usize,
    /// Where more bytes come from as the program reads (contract 0's `Io::Stream`); `None`
    /// once it ends.
    more: Option<Box<dyn std::io::Read + Send>>,
}
impl Input {
    pub fn new(bytes: &[u8]) -> Input {
        Input {
            bytes: bytes.to_vec(),
            at: 0,
            more: None,
        }
    }
    /// Input read from `more` as the program asks for it.
    pub fn streamed(more: Box<dyn std::io::Read + Send>) -> Input {
        Input {
            bytes: Vec::new(),
            at: 0,
            more: Some(more),
        }
    }
    /// Whether byte `i` is in the input, reading more until it is or the input ends. Before it
    /// waits for more, `out` goes to the judge, which may be waiting for it to answer.
    fn has(&mut self, out: &mut Written, i: usize) -> bool {
        while i >= self.bytes.len() {
            let Some(more) = &mut self.more else {
                return false;
            };
            out.flush();
            let mut chunk = [0; 1 << 16];
            match more.read(&mut chunk) {
                Ok(0) => self.more = None,
                Ok(n) => self.bytes.extend_from_slice(&chunk[..n]),
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
                Err(_) => self.more = None,
            }
        }
        true
    }
    /// The next token, past any whitespace. When only spaces, tabs or `\r` follow it on its
    /// line, the line's end is taken too, so a `read_line()` after a line's last token reads the
    /// next line.
    pub fn token(&mut self, out: &mut Written) -> Option<String> {
        let mut s = self.at;
        while self.has(out, s) && is_space(self.bytes[s]) {
            s += 1;
        }
        let mut e = s;
        while self.has(out, e) && !is_space(self.bytes[e]) {
            e += 1;
        }
        let mut k = e;
        while self.has(out, k) && matches!(self.bytes[k], b' ' | b'\t' | b'\r') {
            k += 1;
        }
        self.at = match self.has(out, k).then(|| self.bytes[k]) {
            None => k,
            Some(b'\n') => k + 1,
            Some(_) => e,
        };
        (s < e).then(|| String::from_utf8_lossy(&self.bytes[s..e]).into_owned())
    }
    /// Whether a token is left.
    pub fn left(&mut self, out: &mut Written) -> bool {
        let mut i = self.at;
        while self.has(out, i) {
            if !is_space(self.bytes[i]) {
                return true;
            }
            i += 1;
        }
        false
    }
    /// The byte `k` past the next one, or `-1` past the end; nothing is taken.
    pub fn peek(&mut self, out: &mut Written, k: i64) -> i64 {
        match usize::try_from(k).ok().and_then(|k| self.at.checked_add(k)) {
            Some(i) if self.has(out, i) => i64::from(self.bytes[i]),
            _ => -1,
        }
    }
    /// The next byte, or `-1` at the end.
    pub fn byte(&mut self, out: &mut Written) -> i64 {
        let c = match self.has(out, self.at) {
            true => i64::from(self.bytes[self.at]),
            false => -1,
        };
        self.at += usize::from(c >= 0);
        c
    }
}

/// Whether `op` computes a scalar from scalars, so `scalar` gives its meaning.
pub fn on_scalars(op: Intr) -> bool {
    use Intr as I;
    matches!(
        op,
        I::Add
            | I::Sub
            | I::Mul
            | I::Div
            | I::Rem
            | I::Pow
            | I::Shl
            | I::Shr
            | I::BitAnd
            | I::BitOr
            | I::BitXor
            | I::AddWrap
            | I::SubWrap
            | I::MulWrap
            | I::PowWrap
            | I::ShlWrap
            | I::Neg
            | I::Abs
            | I::Not
            | I::Eq
            | I::Ne
            | I::Lt
            | I::Le
            | I::Gt
            | I::Ge
            | I::Below
            | I::ToU64
            | I::ToF64
            | I::FromU64
            | I::Float(_)
            | I::ToInt(_)
            | I::Popcount
            | I::Ctz
            | I::Clz
            | I::BitWidth
            | I::BitCeil
            | I::CharOf
            | I::CodeOf
            | I::ToI128
            | I::FromI128
            | I::F64OfI128
            | I::I128OfU64
            | I::U64OfI128
            | I::F64Bits
    )
}

/// Whether `op` compares two values.
pub fn compares(op: Intr) -> bool {
    use Intr as I;
    matches!(op, I::Eq | I::Ne | I::Lt | I::Le | I::Gt | I::Ge | I::Below)
}

/// The kinds of a scalar operation's operands and of its result.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Sig {
    pub args: [Operand; 2],
    pub res: Operand,
}

/// The signature of the scalar operation `op` whose first operand is of kind `first`.
pub fn sig(op: Intr, first: Operand) -> Sig {
    use Intr as I;
    use Operand as O;
    let res = match op {
        _ if compares(op) => O::Bool,
        I::Not => O::Bool,
        I::ToU64 | I::U64OfI128 | I::F64Bits => O::U64,
        I::ToF64 | I::F64OfI128 | I::Float(_) => O::F64,
        I::FromU64
        | I::FromI128
        | I::ToInt(_)
        | I::CodeOf
        | I::Popcount
        | I::Ctz
        | I::Clz
        | I::BitWidth
        | I::BitCeil => O::Int,
        I::ToI128 | I::I128OfU64 => O::I128,
        I::CharOf => O::Char,
        _ => first,
    };
    let second = if matches!(op, I::Shl | I::Shr) {
        O::Int
    } else {
        first
    };
    Sig {
        args: [first, second],
        res,
    }
}

/// The value of kind `k` that register words `w` hold.
pub fn load(k: Operand, w: [u64; 2]) -> Option<Scalar> {
    Some(match k {
        Operand::Int => Scalar::Int(w[0] as i64),
        Operand::U64 => Scalar::U64(w[0]),
        Operand::I128 => Scalar::I128((u128::from(w[0]) | (u128::from(w[1]) << 64)) as i128),
        Operand::F64 => Scalar::F64(w[0]),
        Operand::Bool => Scalar::Bool(w[0] != 0),
        Operand::Char => Scalar::Char(u8::try_from(w[0]).ok()?),
        Operand::Other => return None,
    })
}

/// The register words that hold `s`.
pub fn words(s: Scalar) -> [u64; 2] {
    match s {
        Scalar::Int(i) => [i as u64, 0],
        Scalar::U64(u) | Scalar::F64(u) => [u, 0],
        Scalar::I128(w) => [w as u64, ((w as u128) >> 64) as u64],
        Scalar::Bool(b) => [u64::from(b), 0],
        Scalar::Char(c) => [u64::from(c), 0],
    }
}

/// A scalar operation of one operand, on the register words that hold it.
pub fn unary(op: Intr, sig: Sig, a: [u64; 2]) -> Result<[u64; 2], Fault> {
    let other = || Fault::Internal("an operand of another kind");
    let x = load(sig.args[0], a).ok_or_else(other)?;
    scalar(op, &[x]).ok_or_else(other)?.map(words)
}

/// The one meaning of an operation on scalars, for an evaluator that holds values by kind;
/// `None` when the operands are of kinds `op` does not take.
pub fn scalar(op: Intr, xs: &[Scalar]) -> Option<Result<Scalar, Fault>> {
    use Intr as I;
    use Scalar as S;
    let f = f64::from_bits;
    let fl = |r: Result<f64, Fault>| r.map(|x| S::F64(x.to_bits()));
    let whole = |s: Scalar| match s {
        S::Int(i) => Some(i128::from(i)),
        S::I128(w) => Some(w),
        S::U64(u) => Some(i128::from(u)),
        S::Bool(b) => Some(i128::from(b)),
        _ => None,
    };
    let cmp = compares(op);
    let counts = matches!(op, I::Popcount | I::Ctz | I::Clz | I::BitWidth | I::BitCeil);
    Some(match (op, xs) {
        (_, &[S::F64(a), S::F64(b)]) if cmp => compare_float(op, f(a), f(b)).map(S::Bool),
        (_, &[a, b]) if cmp => compare(op, whole(a)?, whole(b)?).map(S::Bool),
        (_, &[S::F64(a), S::F64(b)]) => fl(float(op, f(a), f(b))),
        (I::Shl | I::Shr, &[S::U64(a), S::Int(b)]) => word(op, a, b as u64).map(S::U64),
        (_, &[S::U64(a), S::U64(b)]) => word(op, a, b).map(S::U64),
        (_, &[S::Int(a), S::Int(b)]) => int(op, a, b).map(S::Int),
        (I::Shl | I::Shr, &[S::I128(a), S::Int(b)]) => wide(op, a, i128::from(b)).map(S::I128),
        (_, &[S::I128(a), S::I128(b)]) => wide(op, a, b).map(S::I128),
        (I::Neg, &[S::I128(w)]) => wneg(w).map(S::I128),
        (I::Abs, &[S::I128(w)]) => wabs(w).map(S::I128),
        (I::ToI128, &[S::Int(i)]) => i128_of(i).map(S::I128),
        (I::I128OfU64, &[S::U64(u)]) => i128_of_u64(u).map(S::I128),
        (I::FromI128, &[S::I128(w)]) => int_of_i128(w).map(S::Int),
        (I::U64OfI128, &[S::I128(w)]) => u64_of_i128(w).map(S::U64),
        (I::F64OfI128, &[S::I128(w)]) => fl(Ok(f64_of_i128(w))),
        (I::Neg, &[S::F64(x)]) => fl(Ok(neg_float(f(x)))),
        (I::Abs, &[S::F64(x)]) => fl(Ok(abs_f64(f(x)))),
        (I::Neg, &[S::Int(i)]) => neg(i).map(S::Int),
        (I::Abs, &[S::Int(i)]) => abs(i).map(S::Int),
        (I::Not, &[S::Bool(b)]) => Ok(S::Bool(not(b))),
        (I::ToU64, &[S::Int(i)]) => Ok(S::U64(u64_of_int(i))),
        (I::ToF64, &[S::Int(i)]) => fl(Ok(f64_of_int(i))),
        (I::FromU64, &[S::U64(u)]) => int_of_u64(u).map(S::Int),
        (I::Float(g), &[S::F64(a)]) => fl(float_fn(g, f(a), 0.0)),
        (I::F64Bits, &[S::F64(x)]) => Ok(S::U64(x)),
        (I::ToInt(r), &[S::F64(x)]) => to_int(r, f(x)).map(S::Int),
        (_, &[S::U64(w)]) if counts => bit_count(op, w).map(S::Int),
        (_, &[S::Int(i)]) if counts => bit_count(op, u64_of_int(i)).map(S::Int),
        (I::CharOf, &[S::Int(i)]) => char_of(i).map(S::Char),
        (I::CodeOf, &[S::Char(c)]) => Ok(S::Int(code_of(c))),
        _ => return None,
    })
}

/// A value `print` writes and an event shows as text.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Scalar {
    Int(i64),
    U64(u64),
    I128(i128),
    /// An `F64`, by its bits.
    F64(u64),
    Bool(bool),
    Char(u8),
}
pub fn show(v: Scalar, out: &mut String) {
    use std::fmt::Write;
    let _ = match v {
        Scalar::Int(i) => write!(out, "{i}"),
        Scalar::U64(u) => write!(out, "{u}"),
        Scalar::I128(w) => write!(out, "{w}"),
        Scalar::F64(b) => write!(out, "{}", f64_text(f64::from_bits(b), 10)),
        Scalar::Bool(b) => write!(out, "{b}"),
        Scalar::Char(c) => write!(out, "{}", char::from(c)),
    };
}

/// `x` with `digits` decimals, as C's `%.*f` writes it (R10.3.4): not a number and the
/// infinities too, so a C++ build prints what the engine prints.
pub fn f64_text(x: f64, digits: i64) -> String {
    if !x.is_finite() {
        let sign = if x.is_sign_negative() { "-" } else { "" };
        return format!("{sign}{}", if x.is_nan() { "nan" } else { "inf" });
    }
    let d = usize::try_from(digits).unwrap_or(0);
    format!("{x:.d$}")
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

/// The machine primitives (decision 653): each back end's own storage, input, output and
/// guard model, not semantics. These are the only `Intr` arms a back end has; every fault,
/// bound, input rule and guard result inside them comes from this module, and every other
/// operation is computed from it (`on_scalars`, `scalar`, `cpp_text`).
pub const MACHINE: [Intr; 19] = [
    Intr::RawEmpty,
    Intr::RawNew,
    Intr::RawGet,
    Intr::RawSet,
    Intr::RawTake,
    Intr::RawCap,
    Intr::Out,
    Intr::Flush,
    Intr::InputLeft,
    Intr::ReadInt,
    Intr::ReadByte,
    Intr::PeekByte,
    Intr::RandWord,
    Intr::Unset,
    Intr::CheckKey,
    Intr::CheckRange,
    Intr::CheckVariant,
    Intr::CheckToken,
    Intr::CheckInput,
];

/// Whether the operation divides: R09.3's `div` check guards it, and where that check is off
/// its result for a zero divisor is 0.
pub fn divides(op: Intr) -> bool {
    matches!(op, Intr::Div | Intr::Rem)
}

/// How many characters of `len` an output of `n` writes: `n` clamped to the storage.
pub fn out_len(n: i64, len: usize) -> usize {
    usize::try_from(n).unwrap_or(0).min(len)
}

/// The answer an operation has from its operands' types alone: an ordinal is always below
/// the length of a type that is not a domain, as such a type is its own integers.
pub fn decided(op: Intr, args: &[Arg<CTy>]) -> Option<bool> {
    match (op, args) {
        (Intr::Below, [_, Arg::Value(n)]) => match &n.kind {
            EK::DomLen(t) if !matches!(t, CTy::Dom(_)) => Some(true),
            _ => None,
        },
        _ => None,
    }
}

/// A check std makes (`key_check`, `range_check`): passes when its operand is true.
pub fn check(op: Intr, ok: bool) -> Result<(), Fault> {
    match op {
        Intr::CheckKey => rules::check_key(ok),
        Intr::CheckRange => rules::check_range(ok),
        Intr::CheckVariant => rules::check_variant(ok),
        Intr::CheckToken => rules::check_token(ok),
        Intr::CheckInput => rules::check_input(ok),
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

/// The kind of an operation's first operand, which picks its C++ (`cpp_text`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Operand {
    Int,
    U64,
    I128,
    F64,
    Bool,
    Char,
    Other,
}
impl Operand {
    /// The kind of a value of type `t`.
    pub fn of(t: &CTy) -> Operand {
        match t {
            CTy::Int | CTy::Dom(_) => Operand::Int,
            CTy::U64 => Operand::U64,
            CTy::I128 => Operand::I128,
            CTy::F64 => Operand::F64,
            CTy::Bool => Operand::Bool,
            CTy::Char => Operand::Char,
            CTy::Lent(t) => Operand::of(t),
            _ => Operand::Other,
        }
    }
    /// The register words a value of this kind takes.
    pub fn words(self) -> usize {
        if self == Operand::I128 {
            2
        } else {
            1
        }
    }
}

/// The C++ expression for `op`: `$0`, `$1` and `$2` stand for its operands, `$*` for all of
/// them joined by commas, and `$T` for the C++ name of its result type.
pub fn cpp_text(op: Intr, first: Operand) -> String {
    use Intr as I;
    let text = |c: Cpp| match c {
        Cpp::Infix(o) => format!("($0 {o} $1)"),
        Cpp::Call(f) => format!("{f}($*)"),
    };
    let wide = first == Operand::I128
        || matches!(
            op,
            I::ToI128 | I::I128OfU64 | I::FromI128 | I::U64OfI128 | I::F64OfI128
        );
    // `F64` arithmetic is IEEE in every engine (R02.1).
    let float = match (first, op) {
        (Operand::F64, I::Add) => Some(Cpp::Infix("+")),
        (Operand::F64, I::Sub) => Some(Cpp::Infix("-")),
        (Operand::F64, I::Mul) => Some(Cpp::Infix("*")),
        (Operand::F64, I::Div) => Some(Cpp::Infix("/")),
        (Operand::F64, I::Neg) => return "(-$0)".into(),
        (Operand::F64, I::Abs) => return "__builtin_fabs($0)".into(),
        _ => None,
    };
    let picked = float
        .or_else(|| cpp_wide(op).filter(|_| wide))
        .or_else(|| cpp(op, first == Operand::U64));
    if let Some(c) = picked {
        return text(c);
    }
    match op {
        I::ToU64 => "((uint64_t)($0))".into(),
        I::ToF64 => "((double)($0))".into(),
        I::F64Bits => "cpl_f64_bits($0)".into(),
        I::Float(f) => format!("{}($*)", f.cpp()),
        I::ToInt(r) => format!("cpl_int_of_f64({}($0))", r.cpp()),
        I::FromU64 => "cpl_int_of_u64($0)".into(),
        I::Popcount => "cpl_popcount((uint64_t)($0))".into(),
        I::Ctz => "cpl_ctz((uint64_t)($0))".into(),
        I::Clz => "cpl_clz((uint64_t)($0))".into(),
        I::BitWidth => "cpl_bit_width((uint64_t)($0))".into(),
        I::BitCeil => "cpl_bit_ceil((int64_t)($0))".into(),
        I::Neg => "cpl_neg($0)".into(),
        I::Abs => "cpl_abs($0)".into(),
        I::Below => "((uint64_t)($0) < (uint64_t)($1))".into(),
        I::Not => "(!$0)".into(),
        I::CharOf => "cpl_char_of($0)".into(),
        I::CodeOf => "((int64_t)($0).code)".into(),
        I::RawEmpty | I::Unset => "$T{}".into(),
        I::RawNew => "cpl_raw_new($0, $1, $W)".into(),
        I::RawGet => "cpl_get($0, $1)".into(),
        I::RawSet => "cpl_set($0, $1, $2)".into(),
        I::RawTake => "cpl_take($0, $1)".into(),
        I::RawCap => "cpl_raw_cap<$T>($0, $W)".into(),
        I::CheckKey => "cpl_check_key($0)".into(),
        I::CheckVariant => "cpl_check_variant($0)".into(),
        I::CheckRange => "cpl_check_range($0)".into(),
        I::ReadInt => "cpl_read()".into(),
        I::InputLeft => "cpl_input_left()".into(),
        I::ReadByte => "cpl_read_byte()".into(),
        I::PeekByte => "cpl_peek_byte($0)".into(),
        I::CheckToken => "cpl_check_token($0)".into(),
        I::CheckInput => "cpl_check_input($0)".into(),
        I::RandWord => "cpl_rand_word()".into(),
        I::Out => "cpl_out($0, $1)".into(),
        I::Flush => "(cpl_capture || fflush(stdout), cpl_unit{})".into(),
        // `cpp` and `cpp_wide` spell the rest, for operands of every kind they take.
        _ => "/* not an operation */".into(),
    }
}

/// The state the program's one random stream starts from, so every engine draws the same words.
pub const RAND_SEED: u64 = 0x9e37_79b9_7f4a_7c15;
const RAND_STEP: u64 = 0x9e37_79b9_7f4a_7c15;
const RAND_MIX: [u64; 2] = [0xbf58_476d_1ce4_e5b9, 0x94d0_49bb_1331_11eb];

/// The next word of a random stream: one splitmix64 step.
pub fn rand_word(state: &mut u64) -> u64 {
    *state = state.wrapping_add(RAND_STEP);
    let z = (*state ^ (*state >> 30)).wrapping_mul(RAND_MIX[0]);
    let z = (z ^ (z >> 27)).wrapping_mul(RAND_MIX[1]);
    z ^ (z >> 31)
}

/// `rand_word` in the C++ runtime, from the same constants.
pub fn cpp_rand() -> String {
    format!(
        "static uint64_t cpl_rand_state = {RAND_SEED:#x}ULL;
static inline uint64_t cpl_rand_word() {{
    uint64_t z = cpl_rand_state += {RAND_STEP:#x}ULL;
    z = (z ^ (z >> 30)) * {:#x}ULL;
    z = (z ^ (z >> 27)) * {:#x}ULL;
    return z ^ (z >> 31);
}}
",
        RAND_MIX[0], RAND_MIX[1]
    )
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

/// A `Domain` fn's operation on its `Self`, which the checker writes out in place (R04.2.15).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DomainOp {
    Count,
    Nth,
    At,
    Counted,
    Ord,
}

/// What a std fn declared `= @intrinsic(id)` is: an IR operation, or a domain's.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Intrinsic {
    Op(Intr),
    Domain(DomainOp),
}

impl Intrinsic {
    /// The row of [`INTRINSICS`] spelled `id`.
    pub fn named(id: &str) -> Option<Intrinsic> {
        INTRINSICS.iter().find(|(n, _)| *n == id).map(|(_, i)| *i)
    }
    /// How `@intrinsic` spells it.
    pub fn id(self) -> &'static str {
        INTRINSICS
            .iter()
            .find(|(_, i)| *i == self)
            .map_or("", |(n, _)| n)
    }
    /// The IR operation it is, unless the checker writes it out.
    pub fn op(self) -> Option<Intr> {
        match self {
            Intrinsic::Op(op) => Some(op),
            Intrinsic::Domain(_) => None,
        }
    }
}

/// The parameter and result types an `@intrinsic(id)` fn must have, `|` between the types one
/// accepts, when `id` is one of the table's primitives (R12.14.2).
pub fn entry_sig(id: &str) -> Option<(&'static [&'static str], &'static str)> {
    PrimOp::from_id(id).map(|p| (p.params(), p.result()))
}

/// Whether `ty` is one of `want`, a cell of an entry's signature.
pub fn entry_fits(ty: &crate::ty::Ty, want: &str) -> bool {
    use crate::ty::Ty;
    cpl_prim::Kind::in_cell(want).any(|k| {
        *ty == match k {
            cpl_prim::Kind::Int => Ty::Int,
            cpl_prim::Kind::U64 => Ty::U64,
            cpl_prim::Kind::I128 => Ty::I128,
            cpl_prim::Kind::F64 => Ty::F64,
        }
    })
}

/// Every intrinsic id a std fn's `= @intrinsic(id)` may name, with the operation it is: the one
/// table of them (R12.14.1). The primitives' ids are `cpl-prim`'s, which the old runtime shares.
pub const INTRINSICS: &[(&str, Intrinsic)] = &[
    ("raw_empty", Intrinsic::Op(Intr::RawEmpty)),
    ("raw_new", Intrinsic::Op(Intr::RawNew)),
    ("raw_get", Intrinsic::Op(Intr::RawGet)),
    ("raw_set", Intrinsic::Op(Intr::RawSet)),
    ("move_out", Intrinsic::Op(Intr::RawTake)),
    ("unset_cells", Intrinsic::Op(Intr::RawCap)),
    ("key_check", Intrinsic::Op(Intr::CheckKey)),
    ("range_check", Intrinsic::Op(Intr::CheckRange)),
    ("value_check", Intrinsic::Op(Intr::CheckVariant)),
    ("char_of", Intrinsic::Op(Intr::CharOf)),
    ("rand_word", Intrinsic::Op(Intr::RandWord)),
    ("read_byte", Intrinsic::Op(Intr::ReadByte)),
    ("read_int", Intrinsic::Op(Intr::ReadInt)),
    ("input_peek", Intrinsic::Op(Intr::PeekByte)),
    ("token_check", Intrinsic::Op(Intr::CheckToken)),
    ("input_check", Intrinsic::Op(Intr::CheckInput)),
    ("input_left", Intrinsic::Op(Intr::InputLeft)),
    ("code_of", Intrinsic::Op(Intr::CodeOf)),
    ("u64_of", Intrinsic::Op(Intr::ToU64)),
    ("int_of_u64", Intrinsic::Op(Intr::FromU64)),
    ("i128_of", Intrinsic::Op(Intr::ToI128)),
    ("int_of_i128", Intrinsic::Op(Intr::FromI128)),
    ("f64_of_i128", Intrinsic::Op(Intr::F64OfI128)),
    ("i128_of_u64", Intrinsic::Op(Intr::I128OfU64)),
    ("u64_of_i128", Intrinsic::Op(Intr::U64OfI128)),
    ("f64_of", Intrinsic::Op(Intr::ToF64)),
    ("f64_bits", Intrinsic::Op(Intr::F64Bits)),
    ("out", Intrinsic::Op(Intr::Out)),
    ("flush", Intrinsic::Op(Intr::Flush)),
    (PrimOp::Popcount.id(), Intrinsic::Op(Intr::Popcount)),
    (PrimOp::Abs.id(), Intrinsic::Op(Intr::Abs)),
    (PrimOp::Ctz.id(), Intrinsic::Op(Intr::Ctz)),
    (PrimOp::Clz.id(), Intrinsic::Op(Intr::Clz)),
    (PrimOp::BitWidth.id(), Intrinsic::Op(Intr::BitWidth)),
    (PrimOp::BitCeil.id(), Intrinsic::Op(Intr::BitCeil)),
    (PrimOp::Sqrt.id(), Intrinsic::Op(Intr::Float(FloatFn::Sqrt))),
    (PrimOp::Sin.id(), Intrinsic::Op(Intr::Float(FloatFn::Sin))),
    (PrimOp::Cos.id(), Intrinsic::Op(Intr::Float(FloatFn::Cos))),
    (PrimOp::Tan.id(), Intrinsic::Op(Intr::Float(FloatFn::Tan))),
    (PrimOp::Asin.id(), Intrinsic::Op(Intr::Float(FloatFn::Asin))),
    (PrimOp::Acos.id(), Intrinsic::Op(Intr::Float(FloatFn::Acos))),
    (
        PrimOp::Atan2.id(),
        Intrinsic::Op(Intr::Float(FloatFn::Atan2)),
    ),
    (
        PrimOp::Hypot.id(),
        Intrinsic::Op(Intr::Float(FloatFn::Hypot)),
    ),
    (PrimOp::Exp.id(), Intrinsic::Op(Intr::Float(FloatFn::Exp))),
    (PrimOp::Ln.id(), Intrinsic::Op(Intr::Float(FloatFn::Ln))),
    (
        PrimOp::Floor.id(),
        Intrinsic::Op(Intr::ToInt(Rounding::Floor)),
    ),
    (
        PrimOp::Ceil.id(),
        Intrinsic::Op(Intr::ToInt(Rounding::Ceil)),
    ),
    (
        PrimOp::Round.id(),
        Intrinsic::Op(Intr::ToInt(Rounding::Round)),
    ),
    (
        PrimOp::Trunc.id(),
        Intrinsic::Op(Intr::ToInt(Rounding::Trunc)),
    ),
    ("dom_count", Intrinsic::Domain(DomainOp::Count)),
    ("dom_nth", Intrinsic::Domain(DomainOp::Nth)),
    ("dom_at", Intrinsic::Domain(DomainOp::At)),
    ("dom_counted", Intrinsic::Domain(DomainOp::Counted)),
    ("dom_ord", Intrinsic::Domain(DomainOp::Ord)),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_intrinsic_has_its_cpp() {
        for &(name, i) in INTRINSICS {
            let Some(op) = i.op() else { continue };
            let text = cpp_text(op, Operand::Int);
            assert!(!text.contains("not an operation"), "{name}: {text}");
        }
    }

    #[test]
    fn a_machine_primitive_is_not_computed_here() {
        for op in MACHINE {
            assert!(!on_scalars(op) && !divides(op), "{op:?}");
        }
        assert_eq!((out_len(-1, 3), out_len(2, 3), out_len(9, 3)), (0, 2, 3));
    }

    #[test]
    fn a_one_operand_op_reads_and_writes_its_words() {
        let at = |op, k| sig(op, k);
        let neg = unary(
            Intr::Neg,
            at(Intr::Neg, Operand::I128),
            words(Scalar::I128(-5)),
        );
        assert_eq!(neg, Ok(words(Scalar::I128(5))));
        let widened = unary(Intr::ToI128, at(Intr::ToI128, Operand::Int), [u64::MAX, 0]);
        assert_eq!(widened, Ok(words(Scalar::I128(-1))));
        assert_eq!(at(Intr::ToI128, Operand::Int).res.words(), 2);
        let bits = 1.5f64.to_bits();
        let read = unary(Intr::F64Bits, at(Intr::F64Bits, Operand::F64), [bits, 0]);
        assert_eq!(read, Ok([bits, 0]));
        let low = unary(Intr::Neg, at(Intr::Neg, Operand::Int), [i64::MIN as u64, 0]);
        assert!(low.is_err());
        let wrong = unary(Intr::CodeOf, at(Intr::CodeOf, Operand::Char), [300, 0]);
        assert!(matches!(wrong, Err(Fault::Internal(_))));
    }

    #[test]
    fn a_token_read_splits_at_ascii_whitespace_only() {
        let got: Vec<&str> = tokens(" 1\t2\n\x0b3\x0c\r\n4\u{a0}5\u{85}6 ").collect();
        assert_eq!(got, ["1", "2", "3", "4\u{a0}5\u{85}6"]);
    }
}
