// expect: R0402
// `todo` (R01.6.7) gives no value: a valued fn's body, a branch and a match arm may be `todo`,
// and it traps only when reached.
fn later() -> Int = todo
fn half(n: Int) -> Int =
    if n % 2 == 0:
        n / 2
    else:
        todo // <- here
fn pick(k: Int) -> Int:
    match k:
        0 => 10
        _ => todo
let skip = false
if skip: print(later())
print(half(8), pick(0))
print(half(3))
