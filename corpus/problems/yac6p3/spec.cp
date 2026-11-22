// yac6p3 (interactive): two hidden cells; a query answers the distance to the nearer one. The four
// corners give four sums and differences; one more query tells which pairing of them is real.
let wire = Wire()
let (n, m) = read<(1..=1_000_000_000, 1..=1_000_000_000)>()
fn near(x: Int, y: Int) -> 0..=2_000_000_000 = wire.ask("? {x} {y}")
let a = near(1, 1) + 2
let b = near(n, 1) - n + 1
let c = near(1, m) - m + 1
let d = n + m - near(n, m)
let y = (a + b) / 2
let x = a - y
if (a + b) % 2 == 0 and 1 <= x and x <= n and 1 <= y and y <= m and near(x, y) == 0:
    print("!", x, y, d - (d - c) / 2, (d - c) / 2)
else:
    print("!", (a + c) / 2, a - (a + c) / 2, (d - b) / 2, d - (d - b) / 2)
