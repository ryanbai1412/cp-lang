//! Printing, as ordinary CP: a type is printed by its `Show` impl, and `print` and `write` take
//! any number of values through the `ShowAll` tuple they pack into (R10.3). Core re-exports
//! `Show`, `print` and `write`, so no module imports them.
//!
//! example: `print(1, "a", [2, 3], sep: ",")`
use std.tuple.{Tuple1, Tuple2, Tuple3, Tuple4, Tuple5, Tuple6, Tuple7, Tuple8, Tuple9, Tuple10, Tuple11, Tuple12}

use std.raw.{raw_get, check_range}
/// Writes the first `n` characters of `text` to standard output.
fn out(text: Raw<Char>, n: Int) = @intrinsic("out")

/// How a value prints, and how an interpolated string formats it (R10.3). `Bool` and `?T` are
/// not `Show` (R10.3.3, R10.3.7). Records and structs whose fields all have it print their fields
/// joined, unless they write their own (R10.3.5).
@derive_fields
pub trait Show:
    /// Appends the value to `out`; `sep`, when the caller gives one, joins a sequence's elements.
    fn write_to(self, mut out: Str, sep: ?Str):
        out.append(self.show())
    fn show(self) -> Str:
        var s = Str(List.new())
        self.write_to(mut s, None)
        s
    /// What follows the value in a sequence printed without `sep:`: a space, or a line for a
    /// value with parts (R10.3.5, R10.3.6).
    fn gap(self, mut out: Str):
        out.push(' ')
    /// Appends it with each `F64` in it given `digits` decimals, when given (`precision:`,
    /// `"{x:.3}"`, R10.3.2, R10.3.4).
    fn write_digits(self, mut out: Str, sep: ?Str, digits: ?Int):
        self.write_to(mut out, sep)

/// `sep`, or the space a tuple's fields are joined by.
fn put_sep(mut out: Str, sep: ?Str):
    if let s = sep: out.append(s)
    else: out.push(' ')

/// Each piece of an interpolated string, in turn, padded on the left to `width` with spaces or,
/// after any sign, zeros (`"{x:6}"`, `"{x:04}"`, `"{x:.3}"`, R10.3.2).
@interpolation
fn put_piece<T: Show>(mut out: Str, x: T, width: Int, zero: Bool, digits: ?Int):
    if width == 0:
        x.write_digits(mut out, None, digits)
        return
    var s = Str(List.new())
    x.write_digits(mut s, None, digits)
    let sign = zero and s.len > 0 and s.chars[0] == '-'
    if sign: out.push('-')
    for i in s.len..width: out.push(if zero: '0' else: ' ')
    var skip = sign
    for c in s:
        if skip: skip = false
        else: out.push(c)

/// Values walked one at a time: the first, and a pack of the rest or `()`; a struct's fields
/// and a variadic tuple's parts show through it.
@pack
pub struct Pack<H, T>:
    head: H
    tail: T

/// The values `print` takes, each `Show`.
trait ShowAll:
    /// Appends them joined by `sep`; one value alone is joined inside by `sep` (`print(xs,
    /// sep: ",")`).
    fn write_all(self, mut out: Str, sep: ?Str, first: Bool, digits: ?Int)
    fn ends(self) -> Bool = false

impl ShowAll for ():
    fn write_all(self, mut out: Str, sep: ?Str, first: Bool, digits: ?Int) = ()
    fn ends(self) -> Bool = true

impl<H: Show, T: ShowAll> ShowAll for Pack<H, T>:
    fn write_all(self, mut out: Str, sep: ?Str, first: Bool, digits: ?Int):
        if first and self.tail.ends():
            self.head.write_digits(mut out, sep, digits)
        else:
            if not first: put_sep(mut out, sep)
            self.head.write_digits(mut out, None, digits)
        self.tail.write_all(mut out, sep, false, digits)

impl<A: Show> ShowAll for Tuple1<A>:
    fn write_all(self, mut out: Str, sep: ?Str, first: Bool, digits: ?Int) =
        Pack(self._0, ()).write_all(mut out, sep, first, digits)

impl<A: Show, B: Show> ShowAll for Tuple2<A, B>:
    fn write_all(self, mut out: Str, sep: ?Str, first: Bool, digits: ?Int) =
        Pack(self._0, Pack(self._1, ())).write_all(mut out, sep, first, digits)

impl<A: Show, B: Show, C: Show> ShowAll for Tuple3<A, B, C>:
    fn write_all(self, mut out: Str, sep: ?Str, first: Bool, digits: ?Int) =
        Pack(self._0, Pack(self._1, Pack(self._2, ()))).write_all(mut out, sep, first, digits)

impl<A: Show, B: Show, C: Show, E: Show> ShowAll for Tuple4<A, B, C, E>:
    fn write_all(self, mut out: Str, sep: ?Str, first: Bool, digits: ?Int) =
        Pack(self._0, Pack(self._1, Pack(self._2, Pack(self._3, ())))).write_all(mut out, sep, first, digits)

impl<A: Show, B: Show, C: Show, E: Show, F: Show> ShowAll for Tuple5<A, B, C, E, F>:
    fn write_all(self, mut out: Str, sep: ?Str, first: Bool, digits: ?Int) =
        Pack(self._0, Pack(self._1, Pack(self._2, Pack(self._3, Pack(self._4, ()))))).write_all(mut out, sep, first, digits)

impl<A: Show, B: Show, C: Show, E: Show, F: Show, G: Show> ShowAll for Tuple6<A, B, C, E, F, G>:
    fn write_all(self, mut out: Str, sep: ?Str, first: Bool, digits: ?Int) =
        Pack(self._0, Pack(self._1, Pack(self._2, Pack(self._3, Pack(self._4, Pack(self._5, ())))))).write_all(mut out, sep, first, digits)

impl<A: Show, B: Show, C: Show, E: Show, F: Show, G: Show, H: Show> ShowAll for Tuple7<A, B, C, E, F, G, H>:
    fn write_all(self, mut out: Str, sep: ?Str, first: Bool, digits: ?Int) =
        Pack(self._0, Pack(self._1, Pack(self._2, Pack(self._3, Pack(self._4, Pack(self._5, Pack(self._6, ()))))))).write_all(mut out, sep, first, digits)

impl<A: Show, B: Show, C: Show, E: Show, F: Show, G: Show, H: Show, J: Show> ShowAll for Tuple8<A, B, C, E, F, G, H, J>:
    fn write_all(self, mut out: Str, sep: ?Str, first: Bool, digits: ?Int) =
        Pack(self._0, Pack(self._1, Pack(self._2, Pack(self._3, Pack(self._4, Pack(self._5, Pack(self._6, Pack(self._7, ())))))))).write_all(mut out, sep, first, digits)

impl<A: Show, B: Show, C: Show, E: Show, F: Show, G: Show, H: Show, J: Show, K: Show> ShowAll for Tuple9<A, B, C, E, F, G, H, J, K>:
    fn write_all(self, mut out: Str, sep: ?Str, first: Bool, digits: ?Int) =
        Pack(self._0, Pack(self._1, Pack(self._2, Pack(self._3, Pack(self._4, Pack(self._5, Pack(self._6, Pack(self._7, Pack(self._8, ()))))))))).write_all(mut out, sep, first, digits)

impl<A: Show, B: Show, C: Show, E: Show, F: Show, G: Show, H: Show, J: Show, K: Show, L: Show> ShowAll for Tuple10<A, B, C, E, F, G, H, J, K, L>:
    fn write_all(self, mut out: Str, sep: ?Str, first: Bool, digits: ?Int) =
        Pack(self._0, Pack(self._1, Pack(self._2, Pack(self._3, Pack(self._4, Pack(self._5, Pack(self._6, Pack(self._7, Pack(self._8, Pack(self._9, ())))))))))).write_all(mut out, sep, first, digits)

impl<A: Show, B: Show, C: Show, E: Show, F: Show, G: Show, H: Show, J: Show, K: Show, L: Show, M: Show> ShowAll for Tuple11<A, B, C, E, F, G, H, J, K, L, M>:
    fn write_all(self, mut out: Str, sep: ?Str, first: Bool, digits: ?Int) =
        Pack(self._0, Pack(self._1, Pack(self._2, Pack(self._3, Pack(self._4, Pack(self._5, Pack(self._6, Pack(self._7, Pack(self._8, Pack(self._9, Pack(self._10, ()))))))))))).write_all(mut out, sep, first, digits)

impl<A: Show, B: Show, C: Show, E: Show, F: Show, G: Show, H: Show, J: Show, K: Show, L: Show, M: Show, N: Show> ShowAll for Tuple12<A, B, C, E, F, G, H, J, K, L, M, N>:
    fn write_all(self, mut out: Str, sep: ?Str, first: Bool, digits: ?Int) =
        Pack(self._0, Pack(self._1, Pack(self._2, Pack(self._3, Pack(self._4, Pack(self._5, Pack(self._6, Pack(self._7, Pack(self._8, Pack(self._9, Pack(self._10, Pack(self._11, ())))))))))))).write_all(mut out, sep, first, digits)

/// The fields of a record or struct, joined by spaces; a sequence of them prints one per line
/// (R10.3.5).
impl<H: Show, T: ShowAll> Show for Pack<H, T>:
    fn write_to(self, mut out: Str, sep: ?Str):
        self.write_all(mut out, None, true, None)
    fn write_digits(self, mut out: Str, sep: ?Str, digits: ?Int):
        self.write_all(mut out, None, true, digits)
    fn gap(self, mut out: Str):
        out.push('\n')

/// `print(x, y)`: the values joined by `sep`, then `end`; `precision: k` gives each `F64` `k`
/// decimals (R10.3.1, R10.3.4).
pub fn print<Ts: ShowAll>(..xs: Ts, sep: ?Str = None, end: Str = "\n", precision: ?Int = None):
    var line = Str(List.new())
    xs.write_all(mut line, sep, true, precision)
    line.append(end)
    out(line.chars.buf, line.len)

/// `print` without the line's end.
pub fn write<Ts: ShowAll>(..xs: Ts, sep: ?Str = None, precision: ?Int = None):
    var line = Str(List.new())
    xs.write_all(mut line, sep, true, precision)
    out(line.chars.buf, line.len)

impl Show for Int:
    fn write_to(self, mut out: Str, sep: ?Str):
        // The magnitude as a `U64`, so the least `Int` has one too.
        if self < 0:
            out.push('-')
            (U64(0) - U64(self)).write_to(mut out, sep)
        else:
            U64(self).write_to(mut out, sep)

impl Show for U64:
    fn write_to(self, mut out: Str, sep: ?Str):
        var v = self
        var p = U64(1)
        while v / p >= U64(10): p = p * U64(10)
        while p > U64(0):
            let d = v / p
            out.push(Char(48 + Int(d)))
            v -= d * p
            p = p / U64(10)

fn f64_len(x: F64, digits: Int) -> Int = @intrinsic("f64_len")
fn f64_text(x: F64, digits: Int) -> Raw<Char> = @intrinsic("f64_text")

impl Show for F64:
    fn write_to(self, mut out: Str, sep: ?Str) = self.write_digits(mut out, sep, None)
    /// Ten decimals unless a precision is given (R10.3.4).
    fn write_digits(self, mut out: Str, sep: ?Str, digits: ?Int):
        var d = 10
        if let k = digits: d = k
        check_range(d in 0..=100)
        let text = f64_text(self, d)
        for i in 0..f64_len(self, d): out.push(raw_get(text, i))

impl Show for Char:
    fn write_to(self, mut out: Str, sep: ?Str):
        out.push(self)
    /// A sequence of `Char` prints with nothing between, like a grid's row (R10.3.6).
    fn gap(self, mut out: Str) = ()

impl Show for Str:
    fn write_to(self, mut out: Str, sep: ?Str):
        out.append(self)

/// A position prints as its integer (R04.2.15).
impl<D: Domain> Show for D:
    fn write_to(self, mut out: Str, sep: ?Str):
        Int(self).write_to(mut out, sep)

impl Show for ():
    fn write_to(self, mut out: Str, sep: ?Str) = ()

impl<T: Show> Show for List<T>:
    fn write_to(self, mut out: Str, sep: ?Str) = self.write_digits(mut out, sep, None)
    fn write_digits(self, mut out: Str, sep: ?Str, digits: ?Int):
        var first = true
        for x in self:
            if not first:
                if let s = sep: out.append(s)
                else: x.gap(mut out)
            x.write_digits(mut out, None, digits)
            first = false
    fn gap(self, mut out: Str):
        out.push('\n')

impl<D: Domain, T: Show> Show for Array<D, T>:
    fn write_to(self, mut out: Str, sep: ?Str) = self.write_digits(mut out, sep, None)
    fn write_digits(self, mut out: Str, sep: ?Str, digits: ?Int):
        for i in 0..D.count:
            let x = raw_get(self.buf, i)
            if i > 0:
                if let s = sep: out.append(s)
                else: x.gap(mut out)
            x.write_digits(mut out, None, digits)
    fn gap(self, mut out: Str):
        out.push('\n')

impl<A: Show> Show for Tuple1<A>:
    fn write_to(self, mut out: Str, sep: ?Str) = self.write_digits(mut out, sep, None)
    fn write_digits(self, mut out: Str, sep: ?Str, digits: ?Int) = self._0.write_digits(mut out, None, digits)

impl<A: Show, B: Show> Show for Tuple2<A, B>:
    fn write_to(self, mut out: Str, sep: ?Str) = self.write_digits(mut out, sep, None)
    fn write_digits(self, mut out: Str, sep: ?Str, digits: ?Int):
        self._0.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._1.write_digits(mut out, None, digits)
    fn gap(self, mut out: Str):
        out.push('\n')

impl<A: Show, B: Show, C: Show> Show for Tuple3<A, B, C>:
    fn write_to(self, mut out: Str, sep: ?Str) = self.write_digits(mut out, sep, None)
    fn write_digits(self, mut out: Str, sep: ?Str, digits: ?Int):
        self._0.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._1.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._2.write_digits(mut out, None, digits)
    fn gap(self, mut out: Str):
        out.push('\n')

impl<A: Show, B: Show, C: Show, E: Show> Show for Tuple4<A, B, C, E>:
    fn write_to(self, mut out: Str, sep: ?Str) = self.write_digits(mut out, sep, None)
    fn write_digits(self, mut out: Str, sep: ?Str, digits: ?Int):
        self._0.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._1.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._2.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._3.write_digits(mut out, None, digits)
    fn gap(self, mut out: Str):
        out.push('\n')

impl<A: Show, B: Show, C: Show, E: Show, F: Show> Show for Tuple5<A, B, C, E, F>:
    fn write_to(self, mut out: Str, sep: ?Str) = self.write_digits(mut out, sep, None)
    fn write_digits(self, mut out: Str, sep: ?Str, digits: ?Int):
        self._0.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._1.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._2.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._3.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._4.write_digits(mut out, None, digits)
    fn gap(self, mut out: Str):
        out.push('\n')

impl<A: Show, B: Show, C: Show, E: Show, F: Show, G: Show> Show for Tuple6<A, B, C, E, F, G>:
    fn write_to(self, mut out: Str, sep: ?Str) = self.write_digits(mut out, sep, None)
    fn write_digits(self, mut out: Str, sep: ?Str, digits: ?Int):
        self._0.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._1.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._2.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._3.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._4.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._5.write_digits(mut out, None, digits)
    fn gap(self, mut out: Str):
        out.push('\n')

impl<A: Show, B: Show, C: Show, E: Show, F: Show, G: Show, H: Show> Show for Tuple7<A, B, C, E, F, G, H>:
    fn write_to(self, mut out: Str, sep: ?Str) = self.write_digits(mut out, sep, None)
    fn write_digits(self, mut out: Str, sep: ?Str, digits: ?Int):
        self._0.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._1.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._2.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._3.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._4.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._5.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._6.write_digits(mut out, None, digits)
    fn gap(self, mut out: Str):
        out.push('\n')

impl<A: Show, B: Show, C: Show, E: Show, F: Show, G: Show, H: Show, J: Show> Show for Tuple8<A, B, C, E, F, G, H, J>:
    fn write_to(self, mut out: Str, sep: ?Str) = self.write_digits(mut out, sep, None)
    fn write_digits(self, mut out: Str, sep: ?Str, digits: ?Int):
        self._0.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._1.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._2.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._3.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._4.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._5.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._6.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._7.write_digits(mut out, None, digits)
    fn gap(self, mut out: Str):
        out.push('\n')

impl<A: Show, B: Show, C: Show, E: Show, F: Show, G: Show, H: Show, J: Show, K: Show> Show for Tuple9<A, B, C, E, F, G, H, J, K>:
    fn write_to(self, mut out: Str, sep: ?Str) = self.write_digits(mut out, sep, None)
    fn write_digits(self, mut out: Str, sep: ?Str, digits: ?Int):
        self._0.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._1.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._2.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._3.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._4.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._5.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._6.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._7.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._8.write_digits(mut out, None, digits)
    fn gap(self, mut out: Str):
        out.push('\n')

impl<A: Show, B: Show, C: Show, E: Show, F: Show, G: Show, H: Show, J: Show, K: Show, L: Show> Show for Tuple10<A, B, C, E, F, G, H, J, K, L>:
    fn write_to(self, mut out: Str, sep: ?Str) = self.write_digits(mut out, sep, None)
    fn write_digits(self, mut out: Str, sep: ?Str, digits: ?Int):
        self._0.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._1.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._2.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._3.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._4.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._5.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._6.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._7.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._8.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._9.write_digits(mut out, None, digits)
    fn gap(self, mut out: Str):
        out.push('\n')

impl<A: Show, B: Show, C: Show, E: Show, F: Show, G: Show, H: Show, J: Show, K: Show, L: Show, M: Show> Show for Tuple11<A, B, C, E, F, G, H, J, K, L, M>:
    fn write_to(self, mut out: Str, sep: ?Str) = self.write_digits(mut out, sep, None)
    fn write_digits(self, mut out: Str, sep: ?Str, digits: ?Int):
        self._0.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._1.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._2.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._3.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._4.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._5.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._6.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._7.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._8.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._9.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._10.write_digits(mut out, None, digits)
    fn gap(self, mut out: Str):
        out.push('\n')

impl<A: Show, B: Show, C: Show, E: Show, F: Show, G: Show, H: Show, J: Show, K: Show, L: Show, M: Show, N: Show> Show for Tuple12<A, B, C, E, F, G, H, J, K, L, M, N>:
    fn write_to(self, mut out: Str, sep: ?Str) = self.write_digits(mut out, sep, None)
    fn write_digits(self, mut out: Str, sep: ?Str, digits: ?Int):
        self._0.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._1.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._2.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._3.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._4.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._5.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._6.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._7.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._8.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._9.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._10.write_digits(mut out, None, digits)
        put_sep(mut out, sep)
        self._11.write_digits(mut out, None, digits)
    fn gap(self, mut out: Str):
        out.push('\n')
