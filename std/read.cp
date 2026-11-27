//! Reading input, as ordinary CP: `read<T>()` is `T.read()`, and each type reads by its `Read`
//! impl over the input's bytes (R10.2). Core re-exports `Read`, `read` and `read_line`, so no
//! module imports them.
//!
//! example: `let (n, k): (Int, Int) = read()`
use std.tuple.{Tuple2, Tuple3, Tuple4}

fn read_int() -> Int = @intrinsic("read_int")

/// A type the input reads, token by token, in order (R10.2.1).
pub trait Read:
    fn read() -> Self

/// `read<T>()`: a `T` from the input, by the type it is read at; one read where a range type is
/// expected is held to the range as the input's contract (R10.2.1, R10.2.2).
@input
pub fn read<T: Read>() -> T = T.read()

impl Read for Int:
    fn read() -> Int = read_int()

fn peek_byte(k: Int) -> Int = @intrinsic("input_peek")
/// Traps R0501 unless `ok`: a token read at the end of the input (R10.2.6).
fn check_token(ok: Bool) = @intrinsic("token_check")
/// Traps R0502 unless `ok`: a value the input holds to a contract (R10.2.2).
fn check_input(ok: Bool) = @intrinsic("input_check")
/// Whether the input has another token.
fn token_left() -> Bool = @intrinsic("input_left")
fn blank(c: Int) -> Bool = c == 32 or c in 9..=13

/// Takes the whitespace before the next token; traps at the end of the input (R10.2.6).
fn skip_blank():
    while blank(peek_byte(0)): read_byte()
    check_token(peek_byte(0) != -1)

/// After a token: when only spaces, tabs or `\r` follow it on its line, the line's end is taken
/// too, so a `read_line()` after a line's last token reads the next line (R10.2.3).
fn end_token():
    var k = 0
    while peek_byte(k) == 32 or peek_byte(k) == 9 or peek_byte(k) == 13: k += 1
    let c = peek_byte(k)
    if c == -1 or c == 10:
        for i in 0..(if c == 10: k + 1 else: k): read_byte()

/// `Str` reads one token (R10.2.1).
impl Read for Str:
    fn read() -> Str:
        skip_blank()
        var s = Str(List.new())
        while peek_byte(0) != -1 and not blank(peek_byte(0)): s.push(Char(read_byte()))
        end_token()
        s

/// `Char` takes the whitespace before it, then one character (R10.2.1).
impl Read for Char:
    fn read() -> Char:
        skip_blank()
        Char(read_byte())

/// `[D]T` reads `D.count` values, in walk order (R10.2.1).
impl<D: Domain, T: Read> Read for Array<D, T>:
    fn read() -> [D]T = read_items(D.count)

/// `n` values read in order.
/// `?T` reads a `T` when a token is left, else is `None` (R10.2.10).
impl<T: Read> Read for Optional<T>:
    fn read() -> ?T = if token_left(): Some(T.read()) else: None

/// `List<T>` reads a count, then that many `T` (R10.2.3).
impl<T: Read> Read for List<T>:
    fn read() -> List<T>:
        let n = read_int()
        check_input(n >= 0)
        read_items(n)

/// `Range<D>` reads two ends `l r` as the inclusive `l..=r`, each a position of `D`, and checks
/// `l <= r` (R10.2.3).
impl<D: Domain> Read for Range<D>:
    fn read() -> Range<D>:
        let lo = read_ord<D>()
        let hi = read_ord<D>()
        check_input(lo <= hi)
        Range(lo, hi - lo + 1, 1)

/// The ordinal of the position of `D` an `Int` token names; traps R0502 outside `D`.
fn read_ord<D: Domain>() -> Int:
    if let v = D.at(read_int()): return v.ord
    check_input(false)
    0

fn read_items<T: Read>(n: Int) -> List<T>:
    var xs = List.new()
    for i in 0..n: xs.push(T.read())
    xs

/// A tuple reads its parts in order (R10.2.1).
impl<A: Read, B: Read> Read for Tuple2<A, B>:
    fn read() -> (A, B):
        let a = A.read()
        (a, B.read())

impl<A: Read, B: Read, C: Read> Read for Tuple3<A, B, C>:
    fn read() -> (A, B, C):
        let a = A.read()
        let b = B.read()
        (a, b, C.read())

impl<A: Read, B: Read, C: Read, E: Read> Read for Tuple4<A, B, C, E>:
    fn read() -> (A, B, C, E):
        let a = A.read()
        let b = B.read()
        let c = C.read()
        (a, b, c, E.read())

fn read_byte() -> Int = @intrinsic("read_byte")

/// `read_line()`: the rest of the line, without its `\n` or a trailing `\r`; after a line's last
/// token it is the next line (R10.2.3, R10.2.8).
pub fn read_line() -> Str:
    var s = Str(List.new())
    var cr = false
    var c = read_byte()
    while c != -1 and c != 10:
        if cr: s.push('\r')
        cr = c == 13
        if not cr: s.push(Char(c))
        c = read_byte()
    s
