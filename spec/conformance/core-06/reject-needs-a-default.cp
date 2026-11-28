// expect: E0505
// rule: 06.4.17
// engines: next
// has: "`@reject` guards a parameter's default"
// `@reject` guards a default, so a parameter with none has nothing for it to refuse.
fn widest<T>(
    xs: List<T>,
    @reject(when: T = Str, "pass `by:`", code: E0303) by: T -> Int, // <- here
) -> Int = by(xs[0])
print(widest([1], by: x => x))
