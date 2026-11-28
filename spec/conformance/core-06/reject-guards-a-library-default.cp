// expect: E0303
// rule: 06.4.17
// engines: next
// has: "a `Str`'s width is its own question: pass `by:`"
// Any library may guard a default with `@reject`: a call taking it where `when` holds is the error, and other element types or a passed `by:` are not.
fn widest<T>(
    xs: List<T>,
    @reject(when: T = Str, "a `Str`'s width is its own question: pass `by:`", code: E0303)
    by: T -> Int = _ => 1,
) -> Int:
    var w = 0
    for x in xs: w = max(w, by(x))
    w
print(widest([1, 2]), widest(["abc"], by: s => s.len))
print(widest(["ab"])) // <- here
