let (n, k): (1..=2e5, 1..=2e5) = read()
let a: [0..n]1..=1e9 = read()
fn pieces(len: Int) -> Int = a.map(x => x / len).fold(+)
print((1..=1e9).last_true(len => pieces(len) >= k) ?? 0)
