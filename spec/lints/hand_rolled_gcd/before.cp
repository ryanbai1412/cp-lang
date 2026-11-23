let (a, b): (1..=1000000000, 1..=1000000000) = read()
fn g(x: Int, y: Int) -> Int:
    if y == 0:
        return x
    return g(y, x % y)
print(g(a, b))
