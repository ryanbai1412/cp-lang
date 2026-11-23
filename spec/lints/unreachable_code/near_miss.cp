// A `return` in one branch leaves the code after the `if` reachable.
fn sign(x: Int) -> Int:
    if x < 0:
        return -1
    print(x)
    return 1
print(sign(3))
