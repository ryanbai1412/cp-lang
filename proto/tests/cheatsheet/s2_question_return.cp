// cp: 0.1 bea787c+dirty
fn maybe(x: Int) -> ?Int:
    if x > 0: return x
    return none
fn g(x: Int) -> ?Int:
    let y = maybe(x)?
    return y + 1
print(g(1), g(0))
