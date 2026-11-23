const MOD = 1_000_000_007
fn apply(x: Int, op: Char, y: Int) -> Int:
    match op:
        '+' => (x + y) % MOD
        _ => x * y % MOD
let q: 1..=2e5 = read()
var x = 1
for _ in 0..q:
    let (op, y): (Char, 0..=1e9) = read()
    x = apply(x, op, y)
print(x)
