//! Answer words for yes/no problems. `print`, `read` and the `io` context are core
//! (spec/core/10-io.md); this module holds the library half.

/// The answer word for `ok`: `io.answers.0` when true, else `io.answers.1`.
///
/// The words default to `("YES", "NO")`; a judge profile or `with io(answers: ("Yes", "No"))`
/// changes them for a scope, and `answers:` overrides them for one call. `Bool` is not `Print`,
/// so `print(ok)` is an error that suggests `print(yes(ok))`. The result is a `Str`, so it also
/// works in interpolation.
///
/// cost: O(1)
/// traps: never
/// example: `yes(true) == "YES"`
/// example: `yes(false) == "NO"`
/// example: `yes(false, answers: ("Possible", "Impossible")) == "Impossible"`
pub fn yes(ok: Bool, answers: ?(Str, Str) = None) -> Str = match answers:
    Some((t, f)) => if ok: t else: f
    None => if ok: io.answers.0 else: io.answers.1

/// Interactive mode. Constructing one (`let wire = Wire()`) makes every read and the program's
/// exit flush stdout first; a program that never builds one never flushes early. O(1).
pub struct Wire():
    /// A handle that puts the program in interactive mode (spec/core/10-io.md §10.4).
    ///
    /// From construction on, every read and every exit flushes stdout first, so a query is never
    /// stuck in the buffer while the program waits for its answer. A program that never constructs a
    /// `Wire` never flushes early. Constructing a second `Wire` is allowed and changes nothing.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `let wire = Wire(); print(1); true`
    pub fn new() -> Wire:
        flush()
        Wire()

    /// Print `query` and a newline, flush, then read one answer as the `T` the call site expects.
    ///
    /// `T` comes from the expected type, as with `read`: `fn ask(x: Int, y: Int) -> Int =
    /// wire.ask("? {x} {y}")`. The answer is read with the same rules as `read<T>()`.
    ///
    /// cost: O(|query|) plus the read of one `T`
    /// traps: when input ends before an answer, or the answer is not a valid `T`
    /// example: `let wire = Wire(); let x: Int = wire.ask("? 1 2"); x == 7` (stdin `7`)
    pub fn ask<T>(self, query: Str) -> T:
        print(query)
        flush()
        read<T>()

// blocked: no end-of-input primitive in CP Lang (GAPS.md io)
/// Every remaining `T` on stdin, for statements that give no count.
///
/// Without `until`, it reads to end of input; a later read then traps as reading past the end.
/// With `until: f`, it stops at the first value `v` with `f(v)` true: the sentinel is consumed
/// and left out, and reading may continue after it (decision 161). If no value matches, it reads
/// to end of input like the plain form.
///
/// cost: O(tokens read)
/// order: input order; a new `List`, keyed from 0
/// traps: when a token is not a valid `T`
/// example: `read_all<Int>() == [3, 1, 2]` (stdin `3 1 2`)
/// example: `read_all<Int>(until: x => x == 0) == [5, 7] and read<Int>() == 9` (stdin `5 7 0 9`)
/// example: `read_all<Int>() == []` (empty stdin)
pub fn read_all<T>(until: ?(T -> Bool) = None) -> List<T> = todo

/// Compile-time I/O settings: 0/1-based counted domains, field separator, `yes` words, and float precision.
pub context io:
    base: 0..=1 = 0
    sep: Str = " "
    answers: (Str, Str) = ("YES", "NO")
    precision: ?Int = None
