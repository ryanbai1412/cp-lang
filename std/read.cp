//! Reading input, as ordinary CP: `read<T>()` is `T.read(Input.stdin())`, and each type reads by
//! its `Read` impl over an `Input`'s bytes (R10.2). Core re-exports `Read`, `Input`, `read` and
//! `read_line`, so no module imports them.
//!
//! example: `let (n, k): (Int, Int) = read()`
use std.tuple.{Tuple2, Tuple3, Tuple4}
use std.io.{io}
use std.raw.{raw_new, raw_set}
use std.domains.{Domain, Ordered}
use std.list.{Array, List, Str}
use std.optional.{Optional, Some}
use std.ranges.{Range}
use std.show.{Show}

/// A type the input reads, token by token, in order, from the `Input` it is given (R10.2.1).
@derive
pub trait Read:
    fn read(mut input: Input) -> Self

/// `read<T>()`: a `T` from the input, by the type it is read at; one read where a range type is
/// expected is held to the range as the input's contract (R10.2.1, R10.2.2). `base:` is what a
/// counted domain's first position reads as, at every part of the read (R10.2.2).
@input
pub fn read<T: Read>(base: Int = io.base, none: ?Int = None) -> T uses io:
    var input = Input.stdin(base, none)
    T.read(mut input)

/// `read<T>(from: s)`: a `T` read with `s` as the whole input, by the same `Read` impls as
/// `read<T>()`; `None` on a bad, out-of-range, missing or leftover token. Standard input is
/// untouched (R10.2.4).
pub fn read<T: Read>(; from: Str, base: Int = io.base, none: ?Int = None) -> ?T uses io:
    var input = Input.of(from, base, none)
    let x = T.read(mut input)
    if input.held(): Some(x) else: None

/// Where reads take their bytes: standard input, or a `Str` read whole by `read(from:)`, with
/// where its reads are and whether one failed, which a read from standard input traps on instead.
pub struct Input:
    text: ?Str
    var at: Int
    var failed: Bool
    /// What a counted domain's first position reads as.
    base: Int
    /// The number a `?T` read takes as its `None` (`none: -1`, R10.2.10).
    none: ?Int

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
fn f64_of(text: Raw<Char>, n: Int) -> F64 = @intrinsic("f64_parse")
fn blank(c: Int) -> Bool = c == 32 or c in 9..=13

/// The integer `s` is, when it is one.
fn number(s: Str) -> ?Int:
    var i = Input.of(s)
    let v = i.int()
    if i.held(): Some(v) else: None

impl Input:
    /// Standard input, whose place the program's one byte source keeps.
    pub fn stdin(base: Int = 0, none: ?Int = None) -> Input = Input(None, 0, false, base, none)

    /// `s` as the whole input.
    pub fn of(s: Str, base: Int = 0, none: ?Int = None) -> Input =
        Input(Some(s.clone()), 0, false, base, none)

    /// A `T` read from here, by its `Read` impl: what an impl reads its parts with.
    @input
    pub fn read<T: Read>(mut self) -> T = T.read(mut self)

    /// The index in `tags` of the variant the next token names (R10.2.1): one `Char` when every
    /// tag is one letter, so a column of them reads packed or spaced, else a whole token, which
    /// names an integer tag when it is that integer (`01` names `1`). None named fails, giving 0.
    pub fn variant(mut self, tags: List<Str>) -> Int:
        var letters = true
        for t in tags:
            let c: Int = if t.len == 1: t[0].into() else: 0
            letters = letters and (c in 65..=90 or c in 97..=122)
        if letters:
            self.skip_blank()
            for i in 0..tags.len:
                let c: Int = tags[i][0].into()
                if self.peek(0) == c:
                    self.byte()
                    self.end_token()
                    return i
        else:
            let t = self.read<Str>()
            for i in 0..tags.len:
                if t == tags[i]: return i
                if let a = number(t):
                    if let b = number(tags[i]):
                        if a == b: return i
        self.token(false)
        0

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

    /// Takes the next token when it is `word`, and says whether it was; traps at the end.
    fn taken(mut self, word: Str) -> Bool:
        self.skip_blank()
        for i in 0..word.len:
            let c: Int = word[i].into()
            if self.peek(i) != c: return false
        let c = self.peek(word.len)
        if c != -1 and not blank(c): return false
        for i in 0..word.len: self.byte()
        self.end_token()
        true

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

    /// The position of `D` an `Int` token names: a counted domain's token is its position plus
    /// the input's base, any other domain's is its integer, never shifted. Outside `D` a read
    /// from standard input traps R0502 and one from a string fails (R10.2.2).
    fn position<D: Ordered>(mut self) -> D:
        let shift = if D.counted: self.base else: 0
        if let v = D.at(self.int() - shift): return v
        self.holds(false)
        D.nth(0)

    /// `n` values read in order.
    fn items<T: Read>(mut self, n: Int) -> List<T>:
        var xs = List.new()
        for i in 0..n: xs.push(self.read<T>())
        xs

impl Read for Int:
    fn read(mut input: Input) -> Int = input.int()

/// A position is held to its domain as the input's contract (R10.2.2), wherever the read is, so
/// `List<1..=3>` reads each item as `1..=3` does.
impl Read for _..=_:
    fn read(mut input: Input) -> Self = input.position<Self>()

impl<D: Ordered> Read for D:
    fn read(mut input: Input) -> D = input.position<D>()

/// `I128` reads one whole number; one past `I128`'s range traps R0502 (R10.2.6).
impl Read for I128:
    fn read(mut input: Input) -> I128:
        input.skip_blank()
        let neg = input.peek(0) == 45
        if neg: input.byte()
        input.token(input.peek(0) in 48..=57)
        // Built as a negative, so the least `I128` reads too.
        let least = (I128(1) << 126) * I128(-2)
        var v = I128(0)
        while input.peek(0) in 48..=57:
            let d = I128(input.byte() - 48)
            let fits = v >= (least + d) / I128(10)
            input.holds(fits)
            if fits: v = v * I128(10) - d
        input.holds(neg or v != least)
        input.end_token()
        if neg or v == least: v else: -v

/// `F64` reads one number: a sign, digits with at most one `.` among them, then `e` or `E`, a
/// sign and digits; the value is the `F64` nearest it (R10.2.6). `inf` and `nan` are not numbers.
impl Read for F64:
    fn read(mut input: Input) -> F64:
        input.skip_blank()
        var k = if input.peek(0) == 45 or input.peek(0) == 43: 1 else: 0
        var digits = 0
        while input.peek(k) in 48..=57:
            k += 1
            digits += 1
        if input.peek(k) == 46:
            k += 1
            while input.peek(k) in 48..=57:
                k += 1
                digits += 1
        var ok = digits > 0
        if ok and (input.peek(k) == 101 or input.peek(k) == 69):
            k += 1
            if input.peek(k) == 45 or input.peek(k) == 43: k += 1
            ok = input.peek(k) in 48..=57
            while input.peek(k) in 48..=57: k += 1
        let c = input.peek(k)
        input.token(ok and (c == -1 or blank(c)))
        if not ok: return 0.0
        var text = raw_new(k, Char(48))
        for i in 0..k: raw_set(mut text, i, Char(input.byte()))
        input.end_token()
        f64_of(text, k)

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

/// `?T` reads a `T` when a token is left, else is `None`; under `none: k` the token `k` is the
/// `None` instead, and the end traps (R10.2.10).
impl<T: Read> Read for Optional<T>:
    fn read(mut input: Input) -> ?T:
        if let k = input.none:
            if input.taken(k.show()): return None
        elif not input.left(): return None
        Some(input.read<T>())

/// `List<T>` reads a count, then that many `T` (R10.2.3).
impl<T: Read> Read for List<T>:
    fn read(mut input: Input) -> List<T>:
        let n = input.int()
        input.holds(n >= 0)
        input.items(n)

/// `Range<D>` reads two ends `l r` as the inclusive `l..=r`, each a position of `D`, and checks
/// `l <= r` (R10.2.3).
impl<D: Ordered> Read for Range<D>:
    fn read(mut input: Input) -> Range<D>:
        let lo = input.position<D>().ord
        let hi = input.position<D>().ord
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
