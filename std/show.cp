//! Printing, as ordinary CP: a type is printed by its `Show` impl, and `print` and `write` take
//! any number of values through the `ShowAll` tuple they pack into (R10.3). Core re-exports
//! `Show`, `print` and `write`, so no module imports them.
//!
//! example: `print(1, "a", [2, 3], sep: ",")`
use std.tuple.{Tuple1, Tuple2, Tuple3, Tuple4, Tuple5, Tuple6, Tuple7, Tuple8, Tuple9, Tuple10, Tuple11, Tuple12}

use std.raw.{raw_get, check_range}
use std.io.{io}
use std.domains.{Domain}
use std.list.{Array, List, Str}
use std.optional.{Some}
/// Writes the first `n` characters of `text` to standard output.

/// How a value prints, and how an interpolated string formats it (R10.3). `Bool` and `?T` are
/// not `Show` (R10.3.3, R10.3.7). Records and structs whose fields all have it print their fields
/// joined, unless they write their own (R10.3.5).
@role(derive_fields)
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
    /// Appends it as `fmt` says: each `F64` with its decimals and each counted position in its
    /// base (`precision:`, `"{x:.3}"`, `io.base`, R10.3.2, R10.3.4).
    fn write_fmt(self, mut out: Str, sep: ?Str, fmt: Fmt):
        self.write_to(mut out, sep)

/// How `print` and interpolation write a value: `digits` decimals for an `F64` when given,
/// the `base` a counted domain's positions print in, and the text of `print`'s `none:` for an
/// absent element (R10.2.2, R10.3.1, R10.3.2, R10.3.4).
pub struct Fmt:
    pub digits: ?Int
    pub base: Int
    pub none: ?Str

/// The caller's `io`, with `digits` decimals when given and `io.precision`'s otherwise.
fn fmt_of(digits: ?Int) -> Fmt uses io:
    if let k = digits: Fmt(Some(k), io.base, None) else: Fmt(io.precision, io.base, None)

/// `fmt_of(digits)`, writing an absent element as `none` is written.
fn fmt_none<N: Show>(digits: ?Int, none: N) -> Fmt uses io:
    let f = fmt_of(digits)
    Fmt(f.digits, f.base, Some(shown(none, f)))

/// `x` as `fmt` writes it.
fn shown<T: Show>(x: T, fmt: Fmt) -> Str:
    var s = Str(List.new())
    x.write_fmt(mut s, None, fmt)
    s

/// `sep`, or the space a tuple's fields are joined by.
fn put_sep(mut out: Str, sep: ?Str):
    if let s = sep: out.append(s)
    else: out.push(' ')

/// Each piece of an interpolated string, in turn, padded on the left to `width` with spaces or,
/// after any sign, zeros (`"{x:6}"`, `"{x:04}"`, `"{x:.3}"`, R10.3.2).
@role(interpolation)
fn put_piece<T: Show>(mut out: Str, x: T, width: Int, zero: Bool, digits: ?Int) uses io:
    let fmt = fmt_of(digits)
    if width == 0:
        x.write_fmt(mut out, None, fmt)
        return
    var s = Str(List.new())
    x.write_fmt(mut s, None, fmt)
    let sign = zero and s.len > 0 and s.chars[0] == '-'
    if sign: out.push('-')
    for i in s.len..width: out.push(if zero: '0' else: ' ')
    var skip = sign
    for c in s:
        if skip: skip = false
        else: out.push(c)

/// Values walked one at a time: the first, and a pack of the rest or `()`; a struct's fields
/// and a variadic tuple's parts show through it.
@role(pack)
pub struct Pack<H, T>:
    head: H
    tail: T

/// `print` without `none:`: each value is `Show`, so a `?T` sequence is refused (R10.3.7).
pub struct Bare
/// `print(xs, none: d)`: a sequence of `?T` is a value too (R10.3.1).
pub struct WithNone

/// One of the values `print` takes: any `Show`, and under `WithNone` a `?T` sequence.
pub trait ShowPart<M>:
    fn write_part(self, mut out: Str, sep: ?Str, fmt: Fmt)

impl<T: Show, M> ShowPart<M> for T:
    fn write_part(self, mut out: Str, sep: ?Str, fmt: Fmt) = self.write_fmt(mut out, sep, fmt)

/// The values `print` takes, each a `ShowPart<M>`.
trait ShowAll<M>:
    /// Appends them joined by `sep`; one value alone is joined inside by `sep` (`print(xs,
    /// sep: ",")`).
    fn write_all(self, mut out: Str, sep: ?Str, first: Bool, fmt: Fmt)
    fn ends(self) -> Bool = false

impl<M> ShowAll<M> for ():
    fn write_all(self, mut out: Str, sep: ?Str, first: Bool, fmt: Fmt) = ()
    fn ends(self) -> Bool = true

impl<H: ShowPart<M>, T: ShowAll<M>, M> ShowAll<M> for Pack<H, T>:
    fn write_all(self, mut out: Str, sep: ?Str, first: Bool, fmt: Fmt):
        if first and self.tail.ends():
            self.head.write_part(mut out, sep, fmt)
        else:
            if not first: put_sep(mut out, sep)
            self.head.write_part(mut out, None, fmt)
        self.tail.write_all(mut out, sep, false, fmt)

impl<A: ShowPart<M>, M> ShowAll<M> for Tuple1<A>:
    fn write_all(self, mut out: Str, sep: ?Str, first: Bool, fmt: Fmt) =
        Pack(self._0, ()).write_all(mut out, sep, first, fmt)

impl<A: ShowPart<M>, B: ShowPart<M>, M> ShowAll<M> for Tuple2<A, B>:
    fn write_all(self, mut out: Str, sep: ?Str, first: Bool, fmt: Fmt) =
        Pack(self._0, Pack(self._1, ())).write_all(mut out, sep, first, fmt)

impl<A: ShowPart<M>, B: ShowPart<M>, C: ShowPart<M>, M> ShowAll<M> for Tuple3<A, B, C>:
    fn write_all(self, mut out: Str, sep: ?Str, first: Bool, fmt: Fmt) =
        Pack(self._0, Pack(self._1, Pack(self._2, ()))).write_all(mut out, sep, first, fmt)

impl<A: ShowPart<M>, B: ShowPart<M>, C: ShowPart<M>, E: ShowPart<M>, M> ShowAll<M> for Tuple4<A, B, C, E>:
    fn write_all(self, mut out: Str, sep: ?Str, first: Bool, fmt: Fmt) =
        Pack(self._0, Pack(self._1, Pack(self._2, Pack(self._3, ())))).write_all(mut out, sep, first, fmt)

impl<A: ShowPart<M>, B: ShowPart<M>, C: ShowPart<M>, E: ShowPart<M>, F: ShowPart<M>, M> ShowAll<M> for Tuple5<A, B, C, E, F>:
    fn write_all(self, mut out: Str, sep: ?Str, first: Bool, fmt: Fmt) =
        Pack(self._0, Pack(self._1, Pack(self._2, Pack(self._3, Pack(self._4, ()))))).write_all(mut out, sep, first, fmt)

impl<A: ShowPart<M>, B: ShowPart<M>, C: ShowPart<M>, E: ShowPart<M>, F: ShowPart<M>, G: ShowPart<M>, M> ShowAll<M> for Tuple6<A, B, C, E, F, G>:
    fn write_all(self, mut out: Str, sep: ?Str, first: Bool, fmt: Fmt) =
        Pack(self._0, Pack(self._1, Pack(self._2, Pack(self._3, Pack(self._4, Pack(self._5, ())))))).write_all(mut out, sep, first, fmt)

impl<A: ShowPart<M>, B: ShowPart<M>, C: ShowPart<M>, E: ShowPart<M>, F: ShowPart<M>, G: ShowPart<M>, H: ShowPart<M>, M> ShowAll<M> for Tuple7<A, B, C, E, F, G, H>:
    fn write_all(self, mut out: Str, sep: ?Str, first: Bool, fmt: Fmt) =
        Pack(self._0, Pack(self._1, Pack(self._2, Pack(self._3, Pack(self._4, Pack(self._5, Pack(self._6, ()))))))).write_all(mut out, sep, first, fmt)

impl<A: ShowPart<M>, B: ShowPart<M>, C: ShowPart<M>, E: ShowPart<M>, F: ShowPart<M>, G: ShowPart<M>, H: ShowPart<M>, J: ShowPart<M>, M> ShowAll<M> for Tuple8<A, B, C, E, F, G, H, J>:
    fn write_all(self, mut out: Str, sep: ?Str, first: Bool, fmt: Fmt) =
        Pack(self._0, Pack(self._1, Pack(self._2, Pack(self._3, Pack(self._4, Pack(self._5, Pack(self._6, Pack(self._7, ())))))))).write_all(mut out, sep, first, fmt)

impl<A: ShowPart<M>, B: ShowPart<M>, C: ShowPart<M>, E: ShowPart<M>, F: ShowPart<M>, G: ShowPart<M>, H: ShowPart<M>, J: ShowPart<M>, K: ShowPart<M>, M> ShowAll<M> for Tuple9<A, B, C, E, F, G, H, J, K>:
    fn write_all(self, mut out: Str, sep: ?Str, first: Bool, fmt: Fmt) =
        Pack(self._0, Pack(self._1, Pack(self._2, Pack(self._3, Pack(self._4, Pack(self._5, Pack(self._6, Pack(self._7, Pack(self._8, ()))))))))).write_all(mut out, sep, first, fmt)

impl<A: ShowPart<M>, B: ShowPart<M>, C: ShowPart<M>, E: ShowPart<M>, F: ShowPart<M>, G: ShowPart<M>, H: ShowPart<M>, J: ShowPart<M>, K: ShowPart<M>, L: ShowPart<M>, M> ShowAll<M> for Tuple10<A, B, C, E, F, G, H, J, K, L>:
    fn write_all(self, mut out: Str, sep: ?Str, first: Bool, fmt: Fmt) =
        Pack(self._0, Pack(self._1, Pack(self._2, Pack(self._3, Pack(self._4, Pack(self._5, Pack(self._6, Pack(self._7, Pack(self._8, Pack(self._9, ())))))))))).write_all(mut out, sep, first, fmt)

impl<A: ShowPart<Q>, B: ShowPart<Q>, C: ShowPart<Q>, E: ShowPart<Q>, F: ShowPart<Q>, G: ShowPart<Q>, H: ShowPart<Q>, J: ShowPart<Q>, K: ShowPart<Q>, L: ShowPart<Q>, M: ShowPart<Q>, Q> ShowAll<Q> for Tuple11<A, B, C, E, F, G, H, J, K, L, M>:
    fn write_all(self, mut out: Str, sep: ?Str, first: Bool, fmt: Fmt) =
        Pack(self._0, Pack(self._1, Pack(self._2, Pack(self._3, Pack(self._4, Pack(self._5, Pack(self._6, Pack(self._7, Pack(self._8, Pack(self._9, Pack(self._10, ()))))))))))).write_all(mut out, sep, first, fmt)

impl<A: ShowPart<Q>, B: ShowPart<Q>, C: ShowPart<Q>, E: ShowPart<Q>, F: ShowPart<Q>, G: ShowPart<Q>, H: ShowPart<Q>, J: ShowPart<Q>, K: ShowPart<Q>, L: ShowPart<Q>, M: ShowPart<Q>, N: ShowPart<Q>, Q> ShowAll<Q> for Tuple12<A, B, C, E, F, G, H, J, K, L, M, N>:
    fn write_all(self, mut out: Str, sep: ?Str, first: Bool, fmt: Fmt) =
        Pack(self._0, Pack(self._1, Pack(self._2, Pack(self._3, Pack(self._4, Pack(self._5, Pack(self._6, Pack(self._7, Pack(self._8, Pack(self._9, Pack(self._10, Pack(self._11, ())))))))))))).write_all(mut out, sep, first, fmt)

/// The fields of a record or struct, joined by spaces; a sequence of them prints one per line
/// (R10.3.5).
impl<H: Show, T: ShowAll<Bare>> Show for Pack<H, T>:
    fn write_to(self, mut out: Str, sep: ?Str):
        self.write_all(mut out, None, true, Fmt(None, 0, None))
    fn write_fmt(self, mut out: Str, sep: ?Str, fmt: Fmt):
        self.write_all(mut out, None, true, fmt)
    fn gap(self, mut out: Str):
        out.push('\n')

/// `print(x, y)`: the values joined by `sep`, then `end`; `precision: k` gives each `F64` `k`
/// decimals (R10.3.1, R10.3.4).
@role(print)
pub fn print<Ts: ShowAll<Bare>>(..xs: Ts, sep: ?Str = None, end: Str = "\n", precision: ?Int = None) uses io:
    put_line(xs, sep, end, fmt_of(precision))

/// `print(xs, none: d)`: each absent element of a `?T` sequence printed as `d` (R10.3.1).
pub fn print<Ts: ShowAll<WithNone>, N: Show>(
    ..xs: Ts, sep: ?Str = None, end: Str = "\n", none: N, precision: ?Int = None) uses io:
    put_line(xs, sep, end, fmt_none(precision, none))

/// `print` without the line's end.
@role(write)
pub fn write<Ts: ShowAll<Bare>>(..xs: Ts, sep: ?Str = None, precision: ?Int = None) uses io:
    put_line(xs, sep, "", fmt_of(precision))

/// What a trace shows of a value: the text `write(x)` gives, which each event keeps instead
/// of printing it (contract 8).
@role(trace)
fn traced<T: Show>(x: T) uses io = write(x)

/// What a trace shows of a `Bool`, which is not `Show` (R10.3): `true` or `false`.
@role(trace)
fn traced_bool(x: Bool) uses io = write(if x: "true" else: "false")

/// What a trace shows of a `?T`, which is not `Show`: `none`, or the value it holds.
@role(trace)
fn traced_option<T: Show>(x: ?T) uses io:
    if let v = x: write(v) else: write("none")

/// `write(xs, none: d)`, as `print(xs, none: d)` without the line's end.
pub fn write<Ts: ShowAll<WithNone>, N: Show>(..xs: Ts, sep: ?Str = None, none: N, precision: ?Int = None) uses io:
    put_line(xs, sep, "", fmt_none(precision, none))

/// Under `none:`, a list or array of `?T` writes each absent element as `fmt.none` (R10.3.1).
impl<T: Show> ShowPart<WithNone> for List<?T>:
    fn write_part(self, mut out: Str, sep: ?Str, fmt: Fmt):
        var first = true
        for x in self:
            put_opt(mut out, x, sep, first, fmt)
            first = false

impl<D: Domain, T: Show> ShowPart<WithNone> for Array<D, ?T>:
    fn write_part(self, mut out: Str, sep: ?Str, fmt: Fmt):
        for i in 0..D.count: put_opt(mut out, self.nth(i), sep, i == 0, fmt)

/// One element of a `?T` sequence, after `sep` or its gap unless it is the first.
fn put_opt<T: Show>(mut out: Str, x: ?T, sep: ?Str, first: Bool, fmt: Fmt):
    if not first:
        if let s = sep: out.append(s)
        elif let v = x: v.gap(mut out)
        else: out.push(' ')
    if let v = x: v.write_fmt(mut out, None, fmt)
    elif let d = fmt.none: out.append(d)

/// The values joined by `sep`, then `end`, written to standard output.
fn put_line<Ts: ShowAll<M>, M>(xs: Ts, sep: ?Str, end: Str, fmt: Fmt):
    var line = Str(List.new())
    xs.write_all(mut line, sep, true, fmt)
    line.append(end)
    line.write_out()

impl Show for Int:
    fn write_to(self, mut out: Str, sep: ?Str):
        // The magnitude as a `U64`, so the least `Int` has one too.
        if self < 0:
            out.push('-')
            (U64(0) - U64(self)).write_to(mut out, sep)
        else:
            U64(self).write_to(mut out, sep)

impl Show for I128:
    fn write_to(self, mut out: Str, sep: ?Str):
        if self >= I128(0):
            i128_digits(self, mut out)
        else:
            // `-(self + 1)` does not overflow, so the least `I128` has its digits too.
            out.push('-')
            let m = -(self + I128(1))
            let carry = m % I128(10) == I128(9)
            let high = if carry: m / I128(10) + I128(1) else: m / I128(10)
            if high > I128(0): i128_digits(high, mut out)
            out.push(Char(if carry: 48 else: 49 + Int(m % I128(10))))

/// The digits of `v >= 0`.
fn i128_digits(v: I128, mut out: Str):
    var p = I128(1)
    while v / p >= I128(10): p = p * I128(10)
    var w = v
    while p > I128(0):
        let d = w / p
        out.push(Char(48 + Int(d)))
        w -= d * p
        p = p / I128(10)

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

fn f64_bits(x: F64) -> U64 = @intrinsic("f64_bits")

/// `x` with `d` decimals, as C's `%.*f` writes it: its exact binary value `m * 2^e`, times
/// `10^d`, rounded half to even, in limbs of 30 bits (R10.3.4).
fn write_f64(mut out: Str, x: F64, d: Int):
    let b = f64_bits(x)
    if b >> 63 == U64(1): out.push('-')
    let top = Int((b >> 52) & U64(2047))
    var m = Int(b & U64(4503599627370495))
    if top == 2047:
        out.append(if m == 0: "inf" else: "nan")
        return
    var e = -1074
    if top > 0:
        m += 4503599627370496
        e = top - 1075
    // Most values written are exact in `Int`: a whole part that fits, and room to take the
    // fraction's decimals one at a time.
    if e >= -59 and e <= 10:
        write_f64_int(mut out, m, e, d)
        return
    var n = [m % 1073741824, m / 1073741824]
    for i in 0..d / 9: limbs_times(mut n, 1000000000)
    limbs_times(mut n, 10 ** (d % 9))
    if e >= 0:
        for i in 0..e / 29: limbs_times(mut n, 1 << 29)
        limbs_times(mut n, 1 << (e % 29))
    else: n = limbs_halved(n, -e)
    var ds = limbs_digits(mut n)
    while ds.len < d + 1: ds.push('0')
    for i in 0..ds.len:
        let j = ds.len - 1 - i
        out.push(ds[j])
        if j == d and d > 0: out.push('.')

/// `m * 2^e` with `d` decimals, for `e` in `-59..=10`: the whole part, then the fraction a
/// decimal at a time, rounded half to even.
fn write_f64_int(mut out: Str, m: Int, e: Int, d: Int):
    let s = if e < 0: -e else: 0
    var whole = if e < 0: m >> s else: m << e
    var f = if e < 0: m - (whole << s) else: 0
    var ds = List.new()
    for i in 0..d:
        f *= 10
        ds.push(f >> s)
        f -= (f >> s) << s
    let last = if d > 0: ds[d - 1] else: whole
    if s > 0 and (f > 1 << (s - 1) or (f == 1 << (s - 1) and last % 2 == 1)):
        var i = d - 1
        while i >= 0 and ds[i] == 9:
            ds[i] = 0
            i -= 1
        if i >= 0: ds[i] += 1
        else: whole += 1
    whole.write_to(mut out, None)
    if d > 0: out.push('.')
    for k in ds: out.push(Char(48 + k))

/// `n *= k`, for `k` below `2^30`.
fn limbs_times(mut n: List<Int>, k: Int):
    var c = 0
    for i in 0..n.len:
        let v = n[i] * k + c
        n[i] = v % 1073741824
        c = v / 1073741824
    if c > 0: n.push(c)

/// `n / 2^s`, rounded half to even.
fn limbs_halved(n: List<Int>, s: Int) -> List<Int>:
    var q = List.new()
    let w = s / 30
    let r = s % 30
    let p = 1 << r
    for i in w..n.len:
        let hi = if i + 1 < n.len: n[i + 1] % p else: 0
        q.push(n[i] / p + hi * (1073741824 / p))
    let half = limb_bit(n, s - 1)
    var below = false
    for i in 0..n.len:
        if i < (s - 1) / 30: below = below or n[i] != 0
        elif i == (s - 1) / 30: below = below or n[i] % (1 << ((s - 1) % 30)) != 0
    let odd = q.len > 0 and q[0] % 2 == 1
    if half and (below or odd):
        var i = 0
        var c = 1
        while c > 0:
            if i == q.len: q.push(0)
            q[i] += 1
            c = q[i] / 1073741824
            q[i] = q[i] % 1073741824
            i += 1
    q

/// Whether bit `i` of `n` is set.
fn limb_bit(n: List<Int>, i: Int) -> Bool = i / 30 < n.len and n[i / 30] / (1 << (i % 30)) % 2 == 1

/// `n`'s decimal digits, lowest first, without leading zeros; `n` is used up.
fn limbs_digits(mut n: List<Int>) -> List<Char>:
    var ds = List.new()
    var left = true
    while left:
        var r = 0
        var i = n.len - 1
        left = false
        while i >= 0:
            let v = r * 1073741824 + n[i]
            n[i] = v / 1000000000
            r = v % 1000000000
            left = left or n[i] != 0
            i -= 1
        for k in 0..9:
            ds.push(Char(48 + r % 10))
            r = r / 10
    while ds.len > 1 and ds[ds.len - 1] == '0': ds.pop()
    ds

impl Show for F64:
    fn write_to(self, mut out: Str, sep: ?Str) = self.write_fmt(mut out, sep, Fmt(None, 0, None))
    /// Ten decimals unless a precision is given (R10.3.4).
    fn write_fmt(self, mut out: Str, sep: ?Str, fmt: Fmt):
        var d = 10
        if let k = fmt.digits: d = k
        check_range(d in 0..=100)
        write_f64(mut out, self, d)

impl Show for Char:
    fn write_to(self, mut out: Str, sep: ?Str):
        out.push(self)
    /// A sequence of `Char` prints with nothing between, like a grid's row (R10.3.6).
    fn gap(self, mut out: Str) = ()

impl Show for Str:
    fn write_to(self, mut out: Str, sep: ?Str):
        out.append(self)

/// A position prints as its integer, a counted domain's in the base `fmt` gives (R04.2.15,
/// R10.3.2).
impl<D: Domain> Show for D:
    fn write_to(self, mut out: Str, sep: ?Str) = self.write_fmt(mut out, sep, Fmt(None, 0, None))
    fn write_fmt(self, mut out: Str, sep: ?Str, fmt: Fmt):
        (Int(self) + (if D.counted: fmt.base else: 0)).write_to(mut out, sep)

impl Show for ():
    fn write_to(self, mut out: Str, sep: ?Str) = ()

impl<T: Show> Show for List<T>:
    fn write_to(self, mut out: Str, sep: ?Str) = self.write_fmt(mut out, sep, Fmt(None, 0, None))
    fn write_fmt(self, mut out: Str, sep: ?Str, fmt: Fmt):
        var first = true
        for x in self:
            if not first:
                if let s = sep: out.append(s)
                else: x.gap(mut out)
            x.write_fmt(mut out, None, fmt)
            first = false
    fn gap(self, mut out: Str):
        out.push('\n')

impl<D: Domain, T: Show> Show for Array<D, T>:
    fn write_to(self, mut out: Str, sep: ?Str) = self.write_fmt(mut out, sep, Fmt(None, 0, None))
    fn write_fmt(self, mut out: Str, sep: ?Str, fmt: Fmt):
        for i in 0..D.count:
            let x = self.nth(i)
            if i > 0:
                if let s = sep: out.append(s)
                else: x.gap(mut out)
            x.write_fmt(mut out, None, fmt)
    fn gap(self, mut out: Str):
        out.push('\n')

impl<A: Show> Show for Tuple1<A>:
    fn write_to(self, mut out: Str, sep: ?Str) = self.write_fmt(mut out, sep, Fmt(None, 0, None))
    fn write_fmt(self, mut out: Str, sep: ?Str, fmt: Fmt) = self._0.write_fmt(mut out, None, fmt)

impl<A: Show, B: Show> Show for Tuple2<A, B>:
    fn write_to(self, mut out: Str, sep: ?Str) = self.write_fmt(mut out, sep, Fmt(None, 0, None))
    fn write_fmt(self, mut out: Str, sep: ?Str, fmt: Fmt):
        self._0.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._1.write_fmt(mut out, None, fmt)
    fn gap(self, mut out: Str):
        out.push('\n')

impl<A: Show, B: Show, C: Show> Show for Tuple3<A, B, C>:
    fn write_to(self, mut out: Str, sep: ?Str) = self.write_fmt(mut out, sep, Fmt(None, 0, None))
    fn write_fmt(self, mut out: Str, sep: ?Str, fmt: Fmt):
        self._0.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._1.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._2.write_fmt(mut out, None, fmt)
    fn gap(self, mut out: Str):
        out.push('\n')

impl<A: Show, B: Show, C: Show, E: Show> Show for Tuple4<A, B, C, E>:
    fn write_to(self, mut out: Str, sep: ?Str) = self.write_fmt(mut out, sep, Fmt(None, 0, None))
    fn write_fmt(self, mut out: Str, sep: ?Str, fmt: Fmt):
        self._0.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._1.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._2.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._3.write_fmt(mut out, None, fmt)
    fn gap(self, mut out: Str):
        out.push('\n')

impl<A: Show, B: Show, C: Show, E: Show, F: Show> Show for Tuple5<A, B, C, E, F>:
    fn write_to(self, mut out: Str, sep: ?Str) = self.write_fmt(mut out, sep, Fmt(None, 0, None))
    fn write_fmt(self, mut out: Str, sep: ?Str, fmt: Fmt):
        self._0.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._1.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._2.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._3.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._4.write_fmt(mut out, None, fmt)
    fn gap(self, mut out: Str):
        out.push('\n')

impl<A: Show, B: Show, C: Show, E: Show, F: Show, G: Show> Show for Tuple6<A, B, C, E, F, G>:
    fn write_to(self, mut out: Str, sep: ?Str) = self.write_fmt(mut out, sep, Fmt(None, 0, None))
    fn write_fmt(self, mut out: Str, sep: ?Str, fmt: Fmt):
        self._0.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._1.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._2.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._3.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._4.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._5.write_fmt(mut out, None, fmt)
    fn gap(self, mut out: Str):
        out.push('\n')

impl<A: Show, B: Show, C: Show, E: Show, F: Show, G: Show, H: Show> Show for Tuple7<A, B, C, E, F, G, H>:
    fn write_to(self, mut out: Str, sep: ?Str) = self.write_fmt(mut out, sep, Fmt(None, 0, None))
    fn write_fmt(self, mut out: Str, sep: ?Str, fmt: Fmt):
        self._0.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._1.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._2.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._3.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._4.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._5.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._6.write_fmt(mut out, None, fmt)
    fn gap(self, mut out: Str):
        out.push('\n')

impl<A: Show, B: Show, C: Show, E: Show, F: Show, G: Show, H: Show, J: Show> Show for Tuple8<A, B, C, E, F, G, H, J>:
    fn write_to(self, mut out: Str, sep: ?Str) = self.write_fmt(mut out, sep, Fmt(None, 0, None))
    fn write_fmt(self, mut out: Str, sep: ?Str, fmt: Fmt):
        self._0.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._1.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._2.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._3.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._4.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._5.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._6.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._7.write_fmt(mut out, None, fmt)
    fn gap(self, mut out: Str):
        out.push('\n')

impl<A: Show, B: Show, C: Show, E: Show, F: Show, G: Show, H: Show, J: Show, K: Show> Show for Tuple9<A, B, C, E, F, G, H, J, K>:
    fn write_to(self, mut out: Str, sep: ?Str) = self.write_fmt(mut out, sep, Fmt(None, 0, None))
    fn write_fmt(self, mut out: Str, sep: ?Str, fmt: Fmt):
        self._0.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._1.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._2.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._3.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._4.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._5.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._6.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._7.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._8.write_fmt(mut out, None, fmt)
    fn gap(self, mut out: Str):
        out.push('\n')

impl<A: Show, B: Show, C: Show, E: Show, F: Show, G: Show, H: Show, J: Show, K: Show, L: Show> Show for Tuple10<A, B, C, E, F, G, H, J, K, L>:
    fn write_to(self, mut out: Str, sep: ?Str) = self.write_fmt(mut out, sep, Fmt(None, 0, None))
    fn write_fmt(self, mut out: Str, sep: ?Str, fmt: Fmt):
        self._0.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._1.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._2.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._3.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._4.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._5.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._6.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._7.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._8.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._9.write_fmt(mut out, None, fmt)
    fn gap(self, mut out: Str):
        out.push('\n')

impl<A: Show, B: Show, C: Show, E: Show, F: Show, G: Show, H: Show, J: Show, K: Show, L: Show, M: Show> Show for Tuple11<A, B, C, E, F, G, H, J, K, L, M>:
    fn write_to(self, mut out: Str, sep: ?Str) = self.write_fmt(mut out, sep, Fmt(None, 0, None))
    fn write_fmt(self, mut out: Str, sep: ?Str, fmt: Fmt):
        self._0.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._1.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._2.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._3.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._4.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._5.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._6.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._7.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._8.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._9.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._10.write_fmt(mut out, None, fmt)
    fn gap(self, mut out: Str):
        out.push('\n')

impl<A: Show, B: Show, C: Show, E: Show, F: Show, G: Show, H: Show, J: Show, K: Show, L: Show, M: Show, N: Show> Show for Tuple12<A, B, C, E, F, G, H, J, K, L, M, N>:
    fn write_to(self, mut out: Str, sep: ?Str) = self.write_fmt(mut out, sep, Fmt(None, 0, None))
    fn write_fmt(self, mut out: Str, sep: ?Str, fmt: Fmt):
        self._0.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._1.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._2.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._3.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._4.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._5.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._6.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._7.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._8.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._9.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._10.write_fmt(mut out, None, fmt)
        put_sep(mut out, sep)
        self._11.write_fmt(mut out, None, fmt)
    fn gap(self, mut out: Str):
        out.push('\n')
