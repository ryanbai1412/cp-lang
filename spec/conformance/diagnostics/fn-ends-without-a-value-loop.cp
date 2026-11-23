// expect: E0409 at 3
// A `for` can end, and so can a `while true` with a `break` out of it; then the fn has no value.
fn first_big(xs: List<Int>) -> Int:
    for x in xs:
        if x > 9: return x
fn spin(x: Int) -> Int:
    while true:
        if x > 0: return x
print(first_big([4, 12]), spin(1))
