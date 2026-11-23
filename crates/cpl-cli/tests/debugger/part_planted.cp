// A triangle number that stops one short, planted for `cpl stress --explain` with an `@alt(f)`.
let n: 1..=9 = read()

fn tri(k: Int) -> Int:
    var s = 0
    for i in 1..k: s += i
    return s

@alt(tri)
fn tri_slow(k: Int) -> Int = k * (k + 1) / 2

print(tri(n))
