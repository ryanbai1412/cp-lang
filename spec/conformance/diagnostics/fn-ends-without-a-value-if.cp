// expect: E0409 at 3
// A fn that gives a value must give one on every path: an `if` with no `else` lets the body reach its end.
fn sign(x: Int) -> Int:
    if x > 0:
        return 1
    elif x < 0:
        return -1
print(sign(3))
