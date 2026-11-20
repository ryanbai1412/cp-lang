// Modular inverse, as it has to be written today: Int only, so inputs stop
// at 1e18 where the judge goes to 2^64. Builds; stress OK against 5034440.cpp
// on coprime inputs.
let x: 1..=1e18 = read()
let m: 2..=1e18 = read() where gcd(x, m) == 1
fn ext_gcd(a: Int, b: Int) -> (g: Int, x: Int, y: Int):
    if b == 0: return (a, 1, 0)
    let r = ext_gcd(b, a % b)
    (r.g, r.y, r.x - a / b * r.y)
print(ext_gcd(x, m).x % m)
