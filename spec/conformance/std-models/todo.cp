// expect: R0402
// model: todo
// `todo` is an expression of any type that traps R0402 where it is reached (R01.6.7): the
// branch not taken holds it and runs fine, and the one taken traps there.
fn half(n: Int) -> Int:
    if n % 2 == 0: return n / 2
    todo  // <- here
print(half(4))
print(half(3))
