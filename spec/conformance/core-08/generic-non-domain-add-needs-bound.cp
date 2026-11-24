// expect: E0606
// rule: 04.3.3
// Only a type parameter used as a domain (`[T]`, `Range<T>`, `T.at`, `for v in T`) moves by `+ k` without a bound; any other `T` still needs `T: Add`.
fn next_of<T>(x: T, xs: List<T>) -> List<T> = xs + [x + 1] // <- here
