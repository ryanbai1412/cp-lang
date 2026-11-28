// expect: E0606
// `T.from_int` is `FromInt`'s, so a `T` with no `FromInt` bound has none (R08.2.2, decision 687).
fn lift<T>(v: Int) -> T = T.from_int(v)
print(lift<Int>(5))
