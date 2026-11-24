// `min(x, V.count)` into `0..=V.count` and `max(min(y, 10), 3)` into
// `3..=10` always fit and store unchecked; `min(z, V.count)` with a
// negative `z` does not, and traps in both engines.
fn clamp(n: 1..=100, x: 0..=1000, y: Int, z: Int) -> Int:
    domain V = 0..n
    var c: 0..=V.count = 0
    c = min(x, V.count)
    var d: 3..=10 = 3
    d = max(min(y, 10), 3)
    print(c, d)
    var e: 0..=V.count = 0
    e = min(z, V.count)
    c + d + e
let n: 1..=100 = read()
let x: 0..=1000 = read()
let y: Int = read()
print(clamp(n, x, y, 5))
print(clamp(n, x, -y, -1))
