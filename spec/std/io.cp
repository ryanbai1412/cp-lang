//! Answer words for yes/no problems. `print`, `read` and the `io` context are core
//! (spec/core/10-io.md); this module holds the library half.

/// O(1). The answer word for `ok`: `io.answers.0` when true, else `io.answers.1` (by default
/// `("YES", "NO")`; a judge profile or `with io(answers: ("Yes", "No"))` changes it). `Bool`
/// is not `Print`, so `print(ok)` is an error that suggests `print(yes(ok))`.
pub fn yes(ok: Bool, answers: ?(Str, Str) = none) -> Str = todo
