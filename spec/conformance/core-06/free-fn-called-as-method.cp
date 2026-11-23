// expect: E0203 at 3
// rule: 06.1.2
// has: "gcd(x, 4)"
// A free function called as a method names the call form: `x.gcd(4)` is `gcd(x, 4)` (rebuild round 34 F7).
let x = 12
print(x.gcd(4))
