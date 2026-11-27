//! Reading input, as ordinary CP: `read<T>()` is `T.read(Input.stdin())`, and each type reads by
//! its `Read` impl over an `Input`'s bytes (R10.2). Core re-exports `Read`, `Input`, `read` and
//! `read_line`, so no module imports them.
//!
//! example: `let (n, k): (Int, Int) = read()`
use std.tuple.{Tuple2, Tuple3, Tuple4}

/// A type the input reads, token by token, in order, from the `Input` it is given (R10.2.1).
pub trait Read:
    fn read(mut input: Input) -> Self

/// `read<T>()`: a `T` from the input, by the type it is read at; one read where a range type is
/// expected is held to the range as the input's contract (R10.2.1, R10.2.2).
@input
pub fn read<T: Read>() -> T:
    var input = Input.stdin()
    T.read(mut input)

/// `read<T>(from: s)`: a `T` read with `s` as the whole input, by the same `Read` impls as
/// `read<T>()`; `None` on a bad, out-of-range, missing or leftover token. Standard input is
/// untouched (R10.2.4).
pub fn read<T: Read>(; from: Str) -> ?T:
    var input = Input.of(from)
    let x = T.read(mut input)
    if input.held(): Some(x) else: None

/// Where reads take their bytes: standard input, or a `Str` read whole by `read(from:)`, with
/// where its reads are and whether one failed, which a read from standard input traps on instead.
pub struct Input:
    text: ?Str
    var at: Int
    var failed: Bool

/// `stdin`, or the string read from with where its reads are: `4 x at 1`.
impl Show for Input:
    fn show(i):
        let Some(t) = i.text else: return "stdin"
        var out = t.show()
        out.append(" at ")
        out.append(i.at.show())
        out

fn stdin_int() -> Int = @intrinsic("read_int")
fn stdin_byte() -> Int = @intrinsic("read_byte")
fn stdin_peek(k: Int) -> Int = @intrinsic("input_peek")
fn stdin_token(ok: Bool) = @intrinsic("token_check")
fn stdin_holds(ok: Bool) = @intrinsic("input_check")
fn stdin_left() -> Bool = @intrinsic("input_left")
fn blank(c: Int) -> Bool = c == 32 or c in 9..=13

impl Input:
    /// Standard input, whose place the program's one byte source keeps.
    pub fn stdin() -> Input = Input(None, 0, false)

    /// `s` as the whole input.
    pub fn of(s: Str) -> Input = Input(Some(s.clone()), 0, false)

    /// A `T` read from here, by its `Read` impl: what an impl reads its parts with.
    @input
    pub fn read<T: Read>(mut self) -> T = T.read(mut self)

    /// Whether every read held and no token is left.
    fn held(self) -> Bool = not self.failed and not self.left()

    /// The next byte, taken; -1 at the end.
    fn byte(mut self) -> Int:
        if let s = self.text:
            if self.at >= s.len: return -1
            self.at += 1
            let c: Int = s[self.at - 1].into()
            return c
        stdin_byte()

    /// The byte `k` past where the reads are; -1 past the end.
    fn peek(self, k: Int) -> Int:
        if let s = self.text:
            if self.at + k >= s.len: return -1
            let c: Int = s[self.at + k].into()
            return c
        stdin_peek(k)

    /// Traps R0501 unless `ok`, from standard input: a token read at the end (R10.2.6).
    fn token(mut self, ok: Bool):
        if self.text == None: stdin_token(ok)
        elif not ok: self.failed = true

    /// Traps R0502 unless `ok`, from standard input: a value held to a contract (R10.2.2).
    fn holds(mut self, ok: Bool):
        if self.text == None: stdin_holds(ok)
        elif not ok: self.failed = true

    /// Whether another token is left.
    fn left(self) -> Bool:
        if self.text == None: return stdin_left()
        var k = 0
        while blank(self.peek(k)): k += 1
        self.peek(k) != -1

    /// Takes the whitespace before the next token; traps at the end (R10.2.6).
    fn skip_blank(mut self):
        while blank(self.peek(0)): self.byte()
        self.token(self.peek(0) != -1)

    /// After a token: when only spaces, tabs or `\r` follow it on its line, the line's end is
    /// taken too, so a `read_line()` after a line's last token reads the next line (R10.2.3).
    fn end_token(mut self):
        var k = 0
        while self.peek(k) == 32 or self.peek(k) == 9 or self.peek(k) == 13: k += 1
        let c = self.peek(k)
        if c == -1 or c == 10:
            for i in 0..(if c == 10: k + 1 else: k): self.byte()

    /// An `Int` token: a sign, then digits, a value in `Int` (R10.2.6).
    fn int(mut self) -> Int:
        if self.text == None: return stdin_int()
        self.skip_blank()
        let neg = self.peek(0) == 45
        if neg or self.peek(0) == 43: self.byte()
        var v = 0
        var digits = 0
        while self.peek(0) in 48..=57:
            let d = self.byte() - 48
            let fits = v < 922337203685477580 or (v == 922337203685477580 and d <= 7)
            self.holds(fits)
            if fits: v = v * 10 + d
            digits += 1
        let c = self.peek(0)
        self.token(digits > 0 and (c == -1 or blank(c)))
        self.end_token()
        if neg: -v else: v

    /// The ordinal of the position of `D` an `Int` token names; traps R0502 outside `D`.
    fn ord<D: Domain>(mut self) -> Int:
        if let v = D.at(self.int()): return v.ord
        self.holds(false)
        0

    /// `n` values read in order.
    fn items<T: Read>(mut self, n: Int) -> List<T>:
        var xs = List.new()
        for i in 0..n: xs.push(self.read<T>())
        xs

impl Read for Int:
    fn read(mut input: Input) -> Int = input.int()

/// A range type's value is held to the range as the input's contract (R10.2.2), wherever the
/// read is: outside it a read from standard input traps R0502 and one from a string fails, so
/// `List<1..=3>` reads each item as `1..=3` does.
impl Read for _..=_:
    fn read(mut input: Input) -> Self:
        if let v = Self.at(input.int()): return v
        input.holds(false)
        Self.nth(0)

/// `Str` reads one token (R10.2.1).
impl Read for Str:
    fn read(mut input: Input) -> Str:
        input.skip_blank()
        var s = Str(List.new())
        while input.peek(0) != -1 and not blank(input.peek(0)): s.push(Char(input.byte()))
        input.end_token()
        s

/// `Char` takes the whitespace before it, then one character (R10.2.1).
impl Read for Char:
    fn read(mut input: Input) -> Char:
        input.skip_blank()
        let c = input.byte()
        // No byte is left only past a failed token check in a read from a string, which is
        // then `None` (R10.2.4).
        Char(if c == -1: 0 else: c)

/// `[D]T` reads `D.count` values, in walk order (R10.2.1).
impl<D: Domain, T: Read> Read for Array<D, T>:
    fn read(mut input: Input) -> [D]T = input.items(D.count)

/// `?T` reads a `T` when a token is left, else is `None` (R10.2.10).
impl<T: Read> Read for Optional<T>:
    fn read(mut input: Input) -> ?T = if input.left(): Some(input.read<T>()) else: None

/// `List<T>` reads a count, then that many `T` (R10.2.3).
impl<T: Read> Read for List<T>:
    fn read(mut input: Input) -> List<T>:
        let n = input.int()
        input.holds(n >= 0)
        input.items(n)

/// `Range<D>` reads two ends `l r` as the inclusive `l..=r`, each a position of `D`, and checks
/// `l <= r` (R10.2.3).
impl<D: Domain> Read for Range<D>:
    fn read(mut input: Input) -> Range<D>:
        let lo = input.ord<D>()
        let hi = input.ord<D>()
        input.holds(lo <= hi)
        Range(lo, hi - lo + 1, 1)

/// A tuple reads its parts in order (R10.2.1).
impl<A: Read, B: Read> Read for Tuple2<A, B>:
    fn read(mut input: Input) -> (A, B):
        let a = input.read<A>()
        (a, input.read<B>())

impl<A: Read, B: Read, C: Read> Read for Tuple3<A, B, C>:
    fn read(mut input: Input) -> (A, B, C):
        let a = input.read<A>()
        let b = input.read<B>()
        (a, b, input.read<C>())

impl<A: Read, B: Read, C: Read, E: Read> Read for Tuple4<A, B, C, E>:
    fn read(mut input: Input) -> (A, B, C, E):
        let a = input.read<A>()
        let b = input.read<B>()
        let c = input.read<C>()
        (a, b, c, input.read<E>())

/// `read_line()`: the rest of the line, without its `\n` or a trailing `\r`; after a line's last
/// token it is the next line (R10.2.3, R10.2.8).
pub fn read_line() -> Str:
    var s = Str(List.new())
    var cr = false
    var c = stdin_byte()
    while c != -1 and c != 10:
        if cr: s.push('\r')
        cr = c == 13
        if not cr: s.push(Char(c))
        c = stdin_byte()
    s
