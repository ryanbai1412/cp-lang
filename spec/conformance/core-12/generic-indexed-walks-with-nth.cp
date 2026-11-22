// expect: ok, output "8 6"
// rule: 12.4.13
fn second<S: Indexed, T>(s: S) -> T = s.nth(1)
let s = [5, 6, 7, 8]
print(second(s[2..]), second(s))
