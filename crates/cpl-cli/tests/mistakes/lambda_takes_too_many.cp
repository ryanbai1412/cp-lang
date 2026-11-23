// here: 5
// no fix: a lambda of two parameters where the fn takes one.
fn apply(f: Int -> Int, x: Int) -> Int:
    return f(x)
print(apply((a, b) => a + b, 1))
