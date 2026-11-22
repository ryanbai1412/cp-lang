//! Answer words for yes/no problems. `print`, `read` and the `io` context are core
//! (spec/core/10-io.md); this module holds the library half.

/// O(1). The answer word for `ok`: `io.answers.0` when true, else `io.answers.1` (by default
/// `("YES", "NO")`; a judge profile or `with io(answers: ("Yes", "No"))` changes it). `Bool`
/// is not `Print`, so `print(ok)` is an error that suggests `print(yes(ok))`.
pub fn yes(ok: Bool, answers: ?(Str, Str) = None) -> Str = todo

/// Interactive mode. Constructing one (`let wire = Wire()`) makes every read and the program's
/// exit flush stdout first; a program that never builds one never flushes early. O(1).
pub struct Wire():
    pub fn new() -> Wire = todo

    /// One query: prints `query` as a line, flushes, and reads the answer as the type the call
    /// site expects. `fn ask(x: Int, y: Int) -> Int = wire.ask("? {x} {y}")`. O(len(query)) plus
    /// the read.
    pub fn ask<T>(self, query: Str) -> T = todo

/// O(tokens). Every remaining value of type `T` until the input ends, for statements whose input
/// has no count (`read_all<Int>()`); reading anything after it is an error. With `until:` it
/// stops at the first value that satisfies it (a sentinel command, `0 0`), consumes that value
/// without returning it, and later reads continue: `read_all<Cmd>(until: c => c == Cmd.End)`.
pub fn read_all<T>(until: ?(T -> Bool) = None) -> List<T> = todo
