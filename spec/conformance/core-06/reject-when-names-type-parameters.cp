// expect: E0505
// rule: 06.4.17
// engines: next
// has: "each `T` a type parameter of the fn"
// A condition says what a type parameter is at the call, so its left side names one.
fn widest<T>(
    xs: List<T>,
    @reject(when: Int = Str, "pass `by:`", code: E0303) by: T -> Int = _ => 1, // <- here
) -> Int = by(xs[0])
print(widest([1]))
